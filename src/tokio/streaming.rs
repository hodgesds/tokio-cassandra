use codec::request;
use codec::response;
use codec::header::ProtocolVersion;
use tokio_service::Service;
use futures::Future;
use tokio_core::reactor::Handle;
use tokio_proto::util::client_proxy::ClientProxy;
use tokio_proto::streaming::{Message, Body};
use tokio_proto::streaming::multiplex::{ClientProto, Frame};
use tokio_proto::TcpClient;
use tokio_core::io::{EasyBuf, Codec, Io, Framed};
use std::io;
use std::net::SocketAddr;
use super::shared::{SimpleRequest, SimpleResponse, perform_handshake};
use super::simple;

/// A dummy to show how streaming would work, when implemented
#[derive(Debug)]
pub struct PartialResultMessage;

/// Streamable responses use the body type, which implements stream, with the streamable response.
/// In our case, this will only be the Result response
/// TODO: fix comment above once things get clearer
#[derive(Debug)]
pub enum StreamingMessage {
    Supported(response::SupportedMessage),
    Result(ResponseStream),
    Ready,
}

#[derive(Debug)]
pub struct Response {
    pub message: StreamingMessage,
}

impl From<Response> for simple::Response {
    fn from(f: Response) -> Self {
        simple::Response { message: f.message.into() }
    }
}

impl From<StreamingMessage> for response::Message {
    fn from(f: StreamingMessage) -> Self {
        use self::StreamingMessage::*;
        match f {
            Ready => response::Message::Ready,
            Supported(msg) => response::Message::Supported(msg),
            Result(_) => panic!("streamable messages are not supported"),
        }
    }
}

type ResponseStream = Body<PartialResultMessage, io::Error>;
type ResponseMessage = Message<Response, ResponseStream>;

type RequestMessage = Message<request::Message, RequestStream>;
type RequestStream = Body<request::Message, io::Error>;


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

type CodecInputFrame = Frame<Response, PartialResultMessage, io::Error>;
type CodecOutputFrame = Frame<request::Message, request::Message, io::Error>;

impl From<SimpleRequest> for CodecOutputFrame {
    fn from(SimpleRequest(id, msg): SimpleRequest) -> Self {
        Frame::Message {
            id: id,
            message: msg,
            body: false,
            solo: true,
        }
    }
}


impl Codec for CqlCodec {
    type In = CodecInputFrame;
    type Out = CodecOutputFrame;
    fn decode(&mut self, _buf: &mut EasyBuf) -> Result<Option<Self::In>, io::Error> {
        unimplemented!()
    }
    fn encode(&mut self, _msg: Self::Out, _buf: &mut Vec<u8>) -> io::Result<()> {
        unimplemented!()
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct CqlProto {
    pub version: ProtocolVersion,
}

impl<T: Io + 'static> ClientProto<T> for CqlProto {
    type Request = request::Message;
    type RequestBody = request::Message;
    type Response = Response;
    type ResponseBody = PartialResultMessage;
    type Error = io::Error;

    /// `Framed<T, LineCodec>` is the return value of `io.framed(LineCodec)`
    type Transport = Framed<T, CqlCodec>;
    type BindTransport = Box<Future<Item = Self::Transport, Error = io::Error>>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        let transport = io.framed(CqlCodec::new(self.version));
        perform_handshake(transport)
    }
}

impl From<CodecInputFrame> for SimpleResponse {
    fn from(f: CodecInputFrame) -> Self {
        match f {
            Frame::Message { id, message, .. } => SimpleResponse(id, message.into()),
            Frame::Error { .. } => {
                // TODO: handle frame errors, or assure they can't happen
                panic!("Cannot handle frame errors right now!")
            }
            Frame::Body { .. } => {
                panic!("Streamed bodies must not happen for the simple responses we expect here")
            }
        }
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
            Message::WithoutBody(res) => res,
            Message::WithBody(_head, bodystream) => {
                Response { message: StreamingMessage::Result(bodystream) }
            }
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
