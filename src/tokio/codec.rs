use codec::request::{self, cql_encode};
use codec::header::ProtocolVersion;
use codec::header::{OpCode, Header};
use codec::response::{self, Result, CqlDecode};

use futures::{Future, Sink, Stream};
use tokio_proto::multiplex::{self, RequestId};
use tokio_core::io::{EasyBuf, Codec, Io, Framed};
use std::io;

#[derive(PartialEq, Debug, Clone)]
enum Machine {
    NeedHeader,
    WithHeader { header: Header, body_len: usize },
}

#[derive(PartialEq, Debug, Clone)]
pub struct CqlCodec {
    state: Machine,
    pub flags: u8,
    version: ProtocolVersion,
}

impl CqlCodec {
    fn new(v: ProtocolVersion) -> Self {
        CqlCodec {
            state: Machine::NeedHeader,
            flags: 0,
            version: v,
        }
    }
}

#[derive(Debug)]
pub struct Response {
    pub header: Header,
    pub message: response::Message,
}

fn match_message(version: ProtocolVersion,
                 code: OpCode,
                 buf: EasyBuf)
                 -> Result<response::Message> {
    use codec::header::OpCode::*;
    Ok(match code {
        Supported => {
            response::Message::Supported(response::SupportedMessage::decode(version, buf)?)
        }
        Ready => response::Message::Ready,
        _ => unimplemented!(),
    })
}

impl Codec for CqlCodec {
    type In = (RequestId, Response);
    type Out = (RequestId, request::Message);
    fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<(RequestId, Response)>> {
        use self::Machine::*;
        match self.state {
            NeedHeader => {
                if buf.len() < Header::encoded_len() {
                    return Ok(None);
                }
                let h = Header::try_from(buf.drain_to(Header::encoded_len())
                        .as_slice()).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
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
                use std::mem;
                let h = match mem::replace(&mut self.state, NeedHeader) {
                    WithHeader { header, .. } => header,
                    _ => unreachable!(),
                };
                /* TODO: implement version mismatch test */
                let code = h.op_code.clone();
                let version = h.version.version;
                Ok(Some((h.stream_id as RequestId,
                         Response {
                             header: h,
                             /* TODO: verify amount of consumed bytes equals the
                                               ones actually parsed */
                             message: match_message(version, code,
                                                    buf.drain_to(body_len))
                                 .map_err(|err| io_err(err))?,
                         })))
            }
        }
    }

    fn encode(&mut self, msg: (RequestId, request::Message), buf: &mut Vec<u8>) -> io::Result<()> {
        let (id, req) = msg;

        cql_encode(self.version,
                   self.flags,
                   id as u16, /* FIXME safe cast */
                   req,
                   buf)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct CqlProto {
    pub version: ProtocolVersion,
}

impl<T: Io + 'static> multiplex::ClientProto<T> for CqlProto {
    type Request = request::Message;
    type Response = Response;
    type Transport = Framed<T, CqlCodec>;
    type BindTransport = Box<Future<Item = Self::Transport, Error = io::Error>>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        let transport = io.framed(CqlCodec::new(self.version));
        let handshake = transport.send((0, request::Message::Options))
            .and_then(|transport| transport.into_future().map_err(|(e, _)| e))
            .and_then(|(res, transport)| interpret_response_to_option(transport, res))
            .and_then(|(transport, startup)| send_startup(transport, startup));
        Box::new(handshake)
    }
}

fn io_err<S>(msg: S) -> io::Error
    where S: Into<Box<::std::error::Error + Send + Sync>>
{
    io::Error::new(io::ErrorKind::Other, msg)
}

fn interpret_response_to_option<T>(transport: Framed<T, CqlCodec>,
                                   res: Option<(u64, Response)>)
                                   -> io::Result<(Framed<T, CqlCodec>, request::StartupMessage)>
    where T: Io + 'static
{
    res.ok_or_else(|| io_err("No reply received upon 'OPTIONS' message"))
        .and_then(|(_id, res)| match res.message {
            response::Message::Supported(msg) => {
                let startup = request::StartupMessage {
                    cql_version: msg.latest_cql_version()
                        .ok_or(io_err("Expected CQL_VERSION to contain at least one version"))?
                        .clone(),
                    compression: None,
                };
                Ok((transport, startup))
            }
            msg => {
                Err(io_err(format!("Expected to receive 'SUPPORTED' message but got {:?}", msg)))
            }
        })
}

fn send_startup<T>(transport: Framed<T, CqlCodec>,
                   startup: request::StartupMessage)
                   -> Box<Future<Error = io::Error, Item = Framed<T, CqlCodec>> + 'static>
    where T: Io + 'static
{
    Box::new(transport.send((0, request::Message::Startup(startup)))
        .and_then(|transport| transport.into_future().map_err(|(e, _)| e))
        .and_then(|(res, transport)| {
            res.ok_or_else(|| io_err("No reply received upon 'STARTUP' message"))
                .map(|(_id, _res)| transport)
        }))
}
