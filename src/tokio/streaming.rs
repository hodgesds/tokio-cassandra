use codec::request;
use codec::header::ProtocolVersion;
use tokio_service::Service;
use futures::Future;
use tokio_core::reactor::Handle;
use tokio_proto::util::client_proxy::ClientProxy;
use tokio_proto::streaming::{Message, Body};
use tokio_proto::streaming::multiplex::{ClientProto, Frame, RequestId};
use tokio_proto::TcpClient;
use tokio_core::io::{EasyBuf, Codec, Io, Framed};
use std::io;
use std::net::SocketAddr;
use super::shared::perform_handshake;
use super::simple;

/// The response type of the streaming protocol
#[derive(Debug)]
pub enum Response {
    Once(simple::Response),
    Stream(ResponseStream),
}

/// Represents a response that arrives in one or more chunks.
type ResponseStream = Body<simple::Response, io::Error>;
type RequestStream = Body<request::Message, io::Error>;

type ResponseMessage = Message<simple::Response, ResponseStream>;
type RequestMessage = Message<request::Message, RequestStream>;


#[derive(PartialEq, Debug, Clone)]
pub struct CqlCodec {
    flags: u8,
    version: ProtocolVersion,
}

impl CqlCodec {
    fn new(v: ProtocolVersion) -> Self {
        CqlCodec {
            flags: 0,
            version: v,
        }
    }
}

impl Codec for CqlCodec {
    type In = Frame<simple::Response, simple::Response, io::Error>;
    type Out = Frame<request::Message, request::Message, io::Error>;
    fn decode(&mut self, _buf: &mut EasyBuf) -> Result<Option<Self::In>, io::Error> {
        unimplemented!()
    }
    fn encode(&mut self, _msg: Self::Out, _buf: &mut Vec<u8>) -> io::Result<()> {
        unimplemented!()
    }
}

//impl From<(RequestId, simple::Response)> for Frame<simple::Response,
// simple::Response, io::Error> {
//    fn from(f: (RequestId, simple::Response)) -> Self {
//        unimplemented!()
//    }
//}

#[derive(PartialEq, Debug, Clone)]
pub struct CqlProto {
    pub version: ProtocolVersion,
}

impl<T: Io + 'static> ClientProto<T> for CqlProto {
    type Request = request::Message;
    type RequestBody = request::Message;
    type Response = simple::Response;
    type ResponseBody = simple::Response;
    type Error = io::Error;

    /// `Framed<T, LineCodec>` is the return value of `io.framed(LineCodec)`
    type Transport = Framed<T, CqlCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;
    //    type BindTransport = Box<Future<Item = Self::Transport, Error = io::Error>>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(CqlCodec::new(self.version)))
        //        let transport = io.framed(CqlCodec::new(self.version));
        //        perform_handshake(transport)
    }
}

pub struct ClientHandle {
    inner: ClientProxy<RequestMessage, ResponseMessage, io::Error>,
}

impl From<request::Message> for RequestMessage {
    fn from(msg: request::Message) -> Self {
        Message::WithoutBody(msg)
    }
}

impl From<ResponseMessage> for Response {
    fn from(msg: ResponseMessage) -> Self {
        match msg {
            Message::WithoutBody(msg) => Response::Once(msg),
            Message::WithBody(_head, body) => Response::Stream(body),
        }
    }
}

impl Service for ClientHandle {
    type Request = request::Message;
    type Response = Response;
    type Error = io::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        self.inner.call(req.into()).map(From::from).boxed()
    }
}

/// Currently acts more like a builder, and the desired semantics are yet to be determined.
pub struct Client {
    pub protocol: CqlProto,
}

impl Client {
    pub fn connect(self,
                   addr: &SocketAddr,
                   handle: &Handle)
                   -> Box<Future<Item = ClientHandle, Error = io::Error>> {
        let ret = TcpClient::new(self.protocol)
            .connect(addr, handle)
            .map(|client_proxy| ClientHandle { inner: client_proxy });
        Box::new(ret)
    }
}
