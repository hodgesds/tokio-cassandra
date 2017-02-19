use codec::request;
use codec::header::ProtocolVersion;
use tokio_service::Service;
use futures::Future;
use tokio_core::reactor::Handle;
use tokio_proto::multiplex;
use tokio_proto::streaming::{Message, Body};
use tokio_proto::streaming::multiplex::{ClientProto, Frame};
use tokio_proto::TcpClient;
use tokio_core::net::TcpStream;
use tokio_core::io::{EasyBuf, Codec, Io, Framed};
use std::io;
use std::net::SocketAddr;
use super::simple;

/// The response type of the streaming protocol
#[derive(Debug)]
pub enum Response {
    Once(simple::Response),
    Stream(ResponseStream),
}

/// Represents a response that arrives in one or more chunks.
type ResponseStream = Body<simple::Response, io::Error>;
type ResponseMessage = Message<simple::Response, ResponseStream>;


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
    fn encode(&mut self, _msg: Self::Out, buf: &mut Vec<u8>) -> io::Result<()> {
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
    type Response = simple::Response;
    type ResponseBody = simple::Response;
    type Error = io::Error;

    /// `Framed<T, LineCodec>` is the return value of `io.framed(LineCodec)`
    type Transport = Framed<T, CqlCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;
    //    type BindTransport = Box<Future<Item = Self::Transport, Error = io::Error>>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        let handshake = io.framed(CqlCodec::new(self.version));
        //        let handshake = transport.send((0, request::Message::Options))
        //            .and_then(|transport| transport.into_future().map_err(|(e, _)| e))
        //            .and_then(|(res, transport)| interpret_response_to_option(transport, res))
        //            .and_then(|(transport, startup)| send_startup(transport, startup));
        //        Box::new(handshake)
        Ok(handshake)
    }
}
//
//
//fn match_message(version: ProtocolVersion,
//                 code: OpCode,
//                 buf: EasyBuf)
//                 -> Result<response::Message> {
//    use codec::header::OpCode::*;
//    Ok(match code {
//        Supported => {
//            response::Message::Supported(response::SupportedMessage::decode(version, buf)?)
//        }
//        Ready => response::Message::Ready,
//        _ => unimplemented!(),
//    })
//}
//
//
//fn io_err<S>(msg: S) -> io::Error
//    where S: Into<Box<::std::error::Error + Send + Sync>>
//{
//    io::Error::new(io::ErrorKind::Other, msg)
//}
//
//fn interpret_response_to_option<T>(transport: Framed<T, CqlCodec>,
//                                   res: Option<(u64, Response)>)
//                                   -> io::Result<(Framed<T, CqlCodec>, request::StartupMessage)>
//    where T: Io + 'static
//{
//    res.ok_or_else(|| io_err("No reply received upon 'OPTIONS' message"))
//        .and_then(|(_id, res)| match res.message {
//            response::Message::Supported(msg) => {
//                let startup = request::StartupMessage {
//                    cql_version: msg.latest_cql_version()
//                        .ok_or(io_err("Expected CQL_VERSION to contain at least one version"))?
//                        .clone(),
//                    compression: None,
//                };
//                Ok((transport, startup))
//            }
//            msg => {
//                Err(io_err(format!("Expected to receive 'SUPPORTED' message but got {:?}", msg)))
//            }
//        })
//}
//
//fn send_startup<T>(transport: Framed<T, CqlCodec>,
//                   startup: request::StartupMessage)
//                   -> Box<Future<Error = io::Error, Item = Framed<T, CqlCodec>> + 'static>
//    where T: Io + 'static
//{
//    Box::new(transport.send((0, request::Message::Startup(startup)))
//        .and_then(|transport| transport.into_future().map_err(|(e, _)| e))
//        .and_then(|(res, transport)| {
//            res.ok_or_else(|| io_err("No reply received upon 'STARTUP' message"))
//                .map(|(_id, _res)| transport)
//        }))
//}
//
//pub struct ClientHandle {
//    inner: multiplex::ClientService<TcpStream, CqlProto>,
//}
//
//pub struct Client {
//    pub protocol: CqlProto,
//}

//impl Client {
//    pub fn connect(self,
//                   addr: &SocketAddr,
//                   handle: &Handle)
//                   -> Box<Future<Item = ClientHandle, Error = io::Error>> {
//        let ret = TcpClient::new(self.protocol)
//            .connect(addr, handle)
//            .map(|_client_service| ClientHandle { inner: _client_service });
//        Box::new(ret)
//    }
//}
//
//impl Service for ClientHandle {
//    type Request = request::Message;
//    type Response = Response;
//    type Error = io::Error;
//    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;
//
//    fn call(&self, req: Self::Request) -> Self::Future {
//        self.inner.call(req).boxed()
//    }
//}
//
//
//use codec::request::cql_encode;
//use codec::header::ProtocolVersion;
//use codec::header::{OpCode, Header};
//use codec::response::{self, Result, CqlDecode};
//
//use futures::{Sink, Stream};
//use tokio_proto::multiplex::RequestId;
//use tokio_core::io::{EasyBuf, Codec, Io, Framed};
//
//#[derive(PartialEq, Debug, Clone)]
//enum Machine {
//    NeedHeader,
//    WithHeader { header: Header, body_len: usize },
//}
//
