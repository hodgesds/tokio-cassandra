use codec::request::{self, cql_encode};
use codec::header::{Direction, ProtocolVersion};
use codec::header::Header;
use codec::response;

use futures::Future;
use tokio_proto::multiplex::{self, RequestId};
use tokio_core::io::{EasyBuf, Codec, Io, Framed};
use std::io;
use super::super::shared::{io_err, decode_complete_message_by_opcode, perform_handshake,
                           SimpleResponse, SimpleRequest};

#[derive(PartialEq, Debug, Clone)]
enum Machine {
    NeedHeader,
    WithHeader { header: Header, body_len: usize },
}

#[derive(PartialEq, Debug, Clone)]
pub struct CqlCodec {
    state: Machine,
    flags: u8,
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

impl Codec for CqlCodec {
    type In = (RequestId, response::Message);
    type Out = (RequestId, request::Message);
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
                use std::mem;
                let h = match mem::replace(&mut self.state, NeedHeader) {
                    WithHeader { header, .. } => header,
                    _ => unreachable!(),
                };
                /* TODO: implement version mismatch test */
                let code = h.op_code.clone();
                let version = h.version.version;
                Ok(Some((h.stream_id as RequestId,
                             /* TODO: verify amount of consumed bytes equals the
                                               ones actually parsed */
                             decode_complete_message_by_opcode(version, code,
                                                    buf.drain_to(body_len))
                                 .map_err(|err| io_err(err))?,
                         )))
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
    type Response = response::Message;
    type Transport = Framed<T, CqlCodec>;
    type BindTransport = Box<Future<Item = Self::Transport, Error = io::Error>>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        let transport = io.framed(CqlCodec::new(self.version));
        perform_handshake(transport)
    }
}

impl From<(RequestId, response::Message)> for SimpleResponse {
    fn from((id, res): (RequestId, response::Message)) -> Self {
        SimpleResponse(id, res)
    }
}

impl From<SimpleRequest> for (RequestId, request::Message) {
    fn from(SimpleRequest(id, res): SimpleRequest) -> Self {
        (id, res)
    }
}
