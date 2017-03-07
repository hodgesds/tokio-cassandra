use codec::request;
use codec::response;
use codec::header::ProtocolVersion;
use codec::authentication::{Authenticator, Credentials};
use codec::primitives::{CqlBytes, CqlFrom};
use tokio_service::Service;
use futures::{future, Future};
use tokio_core::reactor::Handle;
use tokio_proto::util::client_proxy::{Response as ClientProxyResponse, ClientProxy};
use tokio_proto::streaming::Message;
use tokio_proto::streaming::multiplex::{RequestId, ClientProto, Frame};
use tokio_proto::TcpClient;
use tokio_core::io::{Io, Framed};
use std::io;
use std::net::SocketAddr;
use super::ssl;

// FIXME - don't use pub here, fix imports
pub use super::messages::*;
pub use super::error::*;
pub use super::codec::*;

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
fn ssl_client(_protocol: CqlProto,
              _addr: &SocketAddr,
              _handle: &Handle,
              _tls: ssl::Options)
              -> Box<Future<Item = ClientProxy<RequestMessage, ResponseMessage, io::Error>, Error = io::Error>> {
    Box::new(future::err(io_err("Please compile this library with \
                                                     --features=ssl")))
}

#[cfg(feature = "with-openssl")]
fn ssl_client(protocol: CqlProto,
              addr: &SocketAddr,
              handle: &Handle,
              tls: ssl::Options)
              -> Box<Future<Item = ClientProxy<RequestMessage, ResponseMessage, io::Error>, Error = io::Error>> {
    use super::ssl::ssl_client::SslClient;
    Box::new(SslClient::new(protocol, tls).connect(addr, handle))
}

#[derive(Clone, Default)]
pub struct Options {
    pub creds: Option<Credentials>,
    pub tls: Option<ssl::Options>,
}

impl Client {
    pub fn connect(self,
                   addr: &SocketAddr,
                   handle: &Handle,
                   options: Options)
                   -> Box<Future<Item = ClientHandle, Error = Error>> {
        let Options { creds, tls } = options;
        let ret = match tls {
                Some(tls) => ssl_client(self.protocol, addr, handle, tls),
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
