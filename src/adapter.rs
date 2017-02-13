use codec::request::{self, cql_encode};
use codec::response;
use codec::header::Header;
use codec::response::CqlDecode;

use tokio_proto::streaming::multiplex::RequestId;
use tokio_core::io::{EasyBuf, Codec};
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
                use codec::header::OpCode::*;
                use std::mem;
                let h = match mem::replace(&mut self.state, NeedHeader) {
                    WithHeader { header, .. } => header,
                    _ => unreachable!(),
                };
                let code = h.op_code.clone();
                Ok(match code {
                    Supported => {
                        let msg =
                             response::SupportedMessage::decode(buf.drain_to(body_len as usize))
                            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
                        /* TODO: verify amount of consumed bytes equals the
                                  ones actually parsed */

                        Some((h.stream_id as RequestId,
                              Response {
                                  header: h,
                                  message: response::Message::Supported(msg),
                              }))
                    }
                    _ => unimplemented!(),
                })
            }
        }
    }

    fn encode(&mut self, msg: (RequestId, request::Message), buf: &mut Vec<u8>) -> io::Result<()> {
        let (id, req) = msg;

        cql_encode(self.flags, id as u16 /* FIXME safe cast */, req, buf)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
    }
}
