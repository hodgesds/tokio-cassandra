use codec::request::{self, cql_encode};
use codec::response;
use codec::header::{Header, ProtocolVersion, Direction};
use codec::authentication::{Authenticator, Credentials};
use codec::primitives::{CqlBytes, CqlFrom};
use std::path::PathBuf;
use std::fs::{File, OpenOptions};
use tokio_service::Service;
use futures::{future, Future};
use tokio_core::reactor::Handle;
use tokio_proto::util::client_proxy::{Response as ClientProxyResponse, ClientProxy};
use tokio_proto::streaming::{Message, Body};
use tokio_proto::streaming::multiplex::{RequestId, ClientProto, Frame};
use tokio_proto::TcpClient;
use tokio_core::io::{EasyBuf, Codec, Io, Framed};
use std::{io, mem};
use std::io::Write;
use std::net::SocketAddr;
use super::utils::{io_err, decode_complete_message_by_opcode};
use super::ssl;

error_chain!{
    errors{
        CqlError(code: i32, msg: String) {
            description("Cql error message from server")
            display("CQL Server Error({}): {}", code, msg)
        }
        HandshakeError(msg: String)
    }

    foreign_links{
        IoErr(::std::io::Error);
    }
}

/// A chunk of a result - similar to response::ResultMessage, but only a chunk of it
/// TODO: this is just a dummy to show the intent - this is likely to change
#[derive(Debug)]
pub struct ResultChunk;

/// A message representing a partial response
#[derive(Debug)]
pub enum ChunkedMessage {
    Result(ResultChunk),
}

/// Streamable responses use the body type, which implements stream, with the streamable response.
/// In our case, this will only be the Result response
/// TODO: fix comment above once things get clearer
#[derive(Debug)]
pub enum StreamingMessage {
    Supported(response::SupportedMessage),
    Error(response::ErrorMessage),
    Partial(ResponseStream),
    Authenticate(response::AuthenticateMessage),
    AuthSuccess(response::AuthSuccessMessage),
    Ready,
}

impl From<StreamingMessage> for response::Message {
    fn from(f: StreamingMessage) -> Self {
        use self::StreamingMessage::*;
        match f {
            Ready => response::Message::Ready,
            Supported(msg) => response::Message::Supported(msg),
            Error(msg) => response::Message::Error(msg),
            AuthSuccess(msg) => response::Message::AuthSuccess(msg),
            Authenticate(msg) => response::Message::Authenticate(msg),
            Partial(_) => panic!("Partials are not suppported - this is just used during handshake"),
        }
    }
}

impl From<response::Message> for StreamingMessage {
    fn from(f: response::Message) -> Self {
        match f {
            response::Message::Ready => StreamingMessage::Ready,
            response::Message::Supported(msg) => StreamingMessage::Supported(msg),
            response::Message::AuthSuccess(msg) => StreamingMessage::AuthSuccess(msg),
            response::Message::Authenticate(msg) => StreamingMessage::Authenticate(msg),
            response::Message::Error(msg) => StreamingMessage::Error(msg),
            response::Message::Result => unimplemented!(),
        }
    }
}

type ResponseStream = Body<ChunkedMessage, io::Error>;
type ResponseMessage = Message<StreamingMessage, ResponseStream>;

type RequestMessage = Message<request::Message, RequestStream>;
type RequestStream = Body<request::Message, io::Error>;


#[derive(PartialEq, Debug, Clone)]
pub struct CqlCodec {
    state: Machine,
    flags: u8,
    version: ProtocolVersion,
    debug: CqlCodecDebuggingOptions,
}

#[derive(PartialEq, Debug, Clone, Default)]
pub struct CqlCodecDebuggingOptions {
    pub dump_decoded_frames_into: Option<PathBuf>,
    pub dump_encoded_frames_into: Option<PathBuf>,
    pub frames_count: usize,
}

#[derive(PartialEq, Debug, Clone)]
enum Machine {
    NeedHeader,
    WithHeader { header: Header, body_len: usize },
}

impl CqlCodec {
    pub fn new(v: ProtocolVersion, debug: CqlCodecDebuggingOptions) -> Self {
        CqlCodec {
            state: Machine::NeedHeader,
            flags: 0,
            version: v,
            debug: debug,
        }
    }

    fn do_encode_debug(&mut self, buf: &Vec<u8>) -> io::Result<()> {
        if let Some(path) = self.debug.dump_encoded_frames_into.clone() {
            let h = Header::try_from(buf.as_slice()).expect("header encoded at beginning of buf");
            let mut f = open_at(self.debug_path(path, &h))?;
            f.write_all(buf)?;
        }
        Ok(())
    }

    fn debug_path(&mut self, mut path: PathBuf, h: &Header) -> PathBuf {
        path.push(format!("{:02}-{:02x}_{:?}.bytes",
                          self.debug.frames_count,
                          h.op_code.as_u8(),
                          h.op_code));
        self.debug.frames_count += 1;
        path
    }

    fn do_decode_debug(&mut self, h: &Header, buf: &EasyBuf, body_len: usize) -> io::Result<()> {
        if let Some(path) = self.debug.dump_decoded_frames_into.clone() {
            let mut f = open_at(self.debug_path(path, h))?;
            f.write_all(&h.encode().expect("header encode to work")[..])?;
            f.write_all(&buf.as_slice()[..body_len])?;
        }
        Ok(())
    }
}

fn open_at(path: PathBuf) -> io::Result<File> {
    OpenOptions::new()
        .read(false)
        .create(true)
        .write(true)
        .open(&path)
        .map_err(|e| {
            io_err(format!("Failed to open '{}' for writing with error with error: {:?}",
                           path.display(),
                           e))
        })
}

type CodecInputFrame = Frame<StreamingMessage, ChunkedMessage, io::Error>;
type CodecOutputFrame = Frame<request::Message, request::Message, io::Error>;

impl Codec for CqlCodec {
    type In = CodecInputFrame;
    type Out = CodecOutputFrame;
    fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<Self::In>> {
        use self::Machine::*;
        match self.state {
            NeedHeader => {
                if buf.len() < Header::encoded_len() {
                    return Ok(None);
                }
                let h = Header::try_from(buf.drain_to(Header::encoded_len())
                        .as_slice()).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
                assert!(h.version.direction == Direction::Response,
                        "As a client protocol, I can only handle response decoding");
                let len = h.length;
                self.state = WithHeader {
                    header: h,
                    body_len: len as usize,
                };

                return self.decode(buf);
            }
            WithHeader { body_len, .. } => {
                if body_len as usize > buf.len() {
                    return Ok(None);
                }
                let h = match mem::replace(&mut self.state, NeedHeader) {
                    WithHeader { header, .. } => header,
                    _ => unreachable!(),
                };
                self.do_decode_debug(&h, &buf, body_len)?;
                /* TODO: implement version mismatch test */
                let code = h.op_code.clone();
                let version = h.version.version;
                assert_stream_id(h.stream_id);
                let msg = Frame::Message {
                    id: h.stream_id as RequestId,
                    /* TODO: verify amount of consumed bytes equals the ones actually parsed */
                    message: decode_complete_message_by_opcode(version, code, buf.drain_to(body_len))
                        .map_err(io_err)?
                        .into(),
                    body: false,
                    solo: false,
                };
                debug!("decoded msg: {:?}", msg);
                Ok(Some(msg))
            }
        }
    }

    fn encode(&mut self, msg: Self::Out, buf: &mut Vec<u8>) -> io::Result<()> {
        match msg {
            Frame::Message { id, message, .. } => {
                debug!("encoded msg: {:?}", message);
                assert!(buf.len() == 0, "expecting an empty vector here");

                assert_stream_id(id as u16);
                let res = cql_encode(self.version,
                                     self.flags,
                                     id as u16, /* FIXME safe cast */
                                     message,
                                     buf)
                    .map_err(io_err);
                self.do_encode_debug(buf)?;
                res
            }
            Frame::Error { error, .. } => Err(error),
            Frame::Body { .. } => panic!("Streaming of Requests is not currently supported"),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct CqlProto {
    pub version: ProtocolVersion,
    pub debug: Option<CqlCodecDebuggingOptions>,
}

impl<T: Io + 'static> ClientProto<T> for CqlProto {
    type Request = request::Message;
    type RequestBody = request::Message;
    type Response = StreamingMessage;
    type ResponseBody = ChunkedMessage;
    type Error = io::Error;

    /// `Framed<T, LineCodec>` is the return value of `io.framed(LineCodec)`
    type Transport = Framed<T, CqlCodec>;
    type BindTransport = io::Result<Self::Transport>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        debug!("binding transport!");
        Ok(io.framed(CqlCodec::new(self.version, self.debug.clone().unwrap_or_default())))
    }
}


pub struct ClientHandle {
    inner: Box<Service<Request = RequestMessage,
                       Response = ResponseMessage,
                       Error = io::Error,
                       Future = ClientProxyResponse<ResponseMessage, io::Error>>>,
}

impl From<request::Message> for RequestMessage {
    fn from(msg: request::Message) -> Self {
        Message::WithoutBody(msg)
    }
}

impl From<ResponseMessage> for StreamingMessage {
    fn from(msg: ResponseMessage) -> Self {
        match msg {
            Message::WithoutBody(res) => res,
            Message::WithBody(_head, bodystream) => StreamingMessage::Partial(bodystream),
        }
    }
}

impl Service for ClientHandle {
    type Request = request::Message;
    type Response = StreamingMessage;
    type Error = io::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        Box::new(self.inner.call(req.into()).map(From::from))
    }
}

/// Currently acts more like a builder, and the desired semantics are yet to be determined.
pub struct Client {
    pub protocol: CqlProto,
}

#[cfg(not(feature = "with-openssl"))]
fn ssl_client
    (_protocol: CqlProto,
     _addr: &SocketAddr,
     _handle: &Handle,
     _tls: ssl::Options)
     -> Option<Box<Future<Item = ClientProxy<RequestMessage, ResponseMessage, io::Error>, Error = io::Error>>> {
    None
}

#[cfg(feature = "with-openssl")]
fn ssl_client
    (protocol: CqlProto,
     addr: &SocketAddr,
     handle: &Handle,
     tls: ssl::Options)
     -> Option<Box<Future<Item = ClientProxy<RequestMessage, ResponseMessage, io::Error>, Error = io::Error>>> {
    use super::ssl::ssl_client::SslClient;
    Some(Box::new(SslClient::new(protocol, tls).connect(addr, handle)))
}

impl Client {
    pub fn connect(self,
                   addr: &SocketAddr,
                   handle: &Handle,
                   creds: Option<Credentials>,
                   tls: Option<ssl::Options>)
                   -> Box<Future<Item = ClientHandle, Error = Error>> {
        let ret = match tls {
                Some(tls) => {
                    ssl_client(self.protocol, addr, handle, tls).unwrap_or_else(|| {
                        Box::new(future::err(io_err("Please compile this library with \
                                                     --features=ssl")))
                    })
                }
                None => Box::new(TcpClient::new(self.protocol).connect(addr, handle)),
            }
            .map(|client_proxy| ClientHandle { inner: Box::new(client_proxy) })
            .and_then(|client_handle| client_handle.call(request::Message::Options).map(|r| (r, client_handle)))
            .map_err(|e| e.into())
            .and_then(|(res, ch)| interpret_response_and_handle(ch, res, creds))
            .and_then(|ch| Ok(ch));

        Box::new(ret)
    }
}

impl From<CodecInputFrame> for SimpleResponse {
    fn from(f: CodecInputFrame) -> Self {
        match f {
            Frame::Message { id, message, .. } => SimpleResponse(id, message.into()),
            Frame::Error { .. } => panic!("Frame errors cannot happen here - this is only done during the handshake"),
            Frame::Body { .. } => panic!("Streamed bodies must not happen for the simple responses we expect here"),
        }
    }
}

impl From<SimpleRequest> for CodecOutputFrame {
    fn from(SimpleRequest(id, msg): SimpleRequest) -> Self {
        Frame::Message {
            id: id,
            message: msg,
            body: false,
            solo: false,
        }
    }
}

pub struct SimpleResponse(pub RequestId, pub response::Message);
pub struct SimpleRequest(pub RequestId, pub request::Message);

// TODO: prevent infinite recursion on malformed input
fn interpret_response_and_handle(handle: ClientHandle,
                                 res: StreamingMessage,
                                 creds: Option<Credentials>)
                                 -> Box<Future<Item = ClientHandle, Error = Error>> {
    let res: response::Message = res.into();
    match res {
        response::Message::Supported(msg) => {
            let startup = startup_message_from_supported(msg);
            let f = future::done(startup).and_then(|s| handle.call(s).map_err(|e| e.into()).map(|r| (r, handle)));
            Box::new(f.and_then(|(res, ch)| interpret_response_and_handle(ch, res, creds))
                .and_then(|ch| Ok(ch)))
        }
        response::Message::Authenticate(msg) => {
            let auth_response = auth_response_from_authenticate(creds.clone(), msg);
            let f = future::done(auth_response).and_then(|s| handle.call(s).map_err(|e| e.into()).map(|r| (r, handle)));
            Box::new(f.and_then(|(res, ch)| interpret_response_and_handle(ch, res, creds))
                .and_then(|ch| Ok(ch)))
        }
        response::Message::Ready => Box::new(future::ok(handle)),
        response::Message::AuthSuccess(msg) => {
            debug!("Authentication Succeded: {:?}", msg);
            Box::new(future::ok(handle))
        }
        response::Message::Error(msg) => Box::new(future::err(ErrorKind::CqlError(msg.code, msg.text.into()).into())),
        msg => {
            Box::new(future::err(ErrorKind::HandshakeError(format!("Did not expect to receive \
                                                                    the following message {:?}",
                                                                   msg))
                .into()))
        }
    }


}

fn assert_stream_id(id: u16) {
    // TODO This should not be an assertion, but just a result to be returned.
    // The actual goal is to gain control over the domain of our request IDs, which right
    // now is not present when clients use the service call interface.
    // This should only be possible if there are more than i16::max_value() requests in flight!
    assert!(id as i16 > -1,
            "stream-id {} was negative, which makes it a broadcast id with a special meaning",
            id);
}

fn startup_message_from_supported(msg: response::SupportedMessage) -> Result<request::Message> {
    let startup = {
        request::StartupMessage {
            cql_version: msg.latest_cql_version()
                .ok_or(ErrorKind::HandshakeError("Expected CQL_VERSION to contain at least one version".into()))?
                .clone(),
            compression: None,
        }
    };

    debug!("startup message generated: {:?}", startup);
    Ok(request::Message::Startup(startup))
}

fn auth_response_from_authenticate(creds: Option<Credentials>,
                                   msg: response::AuthenticateMessage)
                                   -> Result<request::Message> {
    let creds =
        creds.ok_or(ErrorKind::HandshakeError(format!("No credentials provided but server requires authentication \
                                                      by {}",
                                                     msg.authenticator.as_ref())))?;

    let authenticator = Authenticator::from_name(msg.authenticator.as_ref(), creds).chain_err(|| "Authenticator Err")?;

    let mut buf = Vec::new();
    authenticator.encode_auth_response(&mut buf);

    Ok(request::Message::AuthResponse(request::AuthResponseMessage {
        auth_data: CqlBytes::try_from(buf).chain_err(|| "Message Err")?,
    }))
}
