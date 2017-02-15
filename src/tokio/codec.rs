use codec::request::{self, cql_encode};
use codec::header::{OpCode, Header};
use codec::response::{self, Result, CqlDecode};

use futures::{Future, Sink, IntoFuture, Stream};
use tokio_proto::multiplex::{self, RequestId};
use tokio_core::io::{EasyBuf, Codec, Io, Framed};
use std::io;

enum Machine {
    NeedHeader,
    WithHeader { header: Header, body_len: usize },
}

pub struct CqlCodecV3 {
    state: Machine,
    pub flags: u8,
}

impl Default for CqlCodecV3 {
    fn default() -> Self {
        CqlCodecV3 {
            state: Machine::NeedHeader,
            flags: 0,
        }
    }
}

#[derive(Debug)]
pub struct Response {
    pub header: Header,
    pub message: response::Message,
}

fn match_message(code: OpCode, buf: EasyBuf) -> Result<response::Message> {
    use codec::header::OpCode::*;
    Ok(match code {
        Supported => response::Message::Supported(response::SupportedMessage::decode(buf)?),
        Ready => response::Message::Ready,
        _ => unimplemented!(),
    })
}

impl Codec for CqlCodecV3 {
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
                let code = h.op_code.clone();
                Ok(Some((h.stream_id as RequestId,
                         Response {
                             header: h,
                             /* TODO: verify amount of consumed bytes equals the
                                               ones actually parsed */
                             message: match_message(code, buf.drain_to(body_len))
                                 .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?,
                         })))
            }
        }
    }

    fn encode(&mut self, msg: (RequestId, request::Message), buf: &mut Vec<u8>) -> io::Result<()> {
        let (id, req) = msg;

        cql_encode(self.flags, id as u16 /* FIXME safe cast */, req, buf)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
    }
}

pub struct CqlProtoV3;

impl<T: Io + 'static> multiplex::ClientProto<T> for CqlProtoV3 {
    type Request = request::Message;
    type Response = Response;
    type Transport = Framed<T, CqlCodecV3>;
    type BindTransport = Box<Future<Item = Self::Transport, Error = io::Error>>;


    fn bind_transport(&self, io: T) -> Self::BindTransport {
        let transport = io.framed(CqlCodecV3::default());
        let handshake = transport.send((0, request::Message::Options))
            .and_then(|transport| transport.into_future().map_err(|(e, _)| e))
            .and_then(|(res, transport)| Ok(transport));
        Box::new(handshake)
    }
}
