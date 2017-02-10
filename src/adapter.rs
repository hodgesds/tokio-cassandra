use codec::request::{self, cql_encode};
use codec::response::{self, CqlDecode};
use codec::header::Header;

use tokio_proto::streaming::multiplex::RequestId;
use tokio_core::io::{EasyBuf, Codec};
use std::io;

enum Machine {
    NeedHeader,
    WithHeader(Header, usize),
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
pub struct Response<'a> {
    store: EasyBuf,
    pub header: Header,
    pub message: response::Message<'a>,
}


impl Codec for CqlCodecV3 {
    type In = (RequestId, Response<'static>);
    type Out = (RequestId, request::Message<'static>);
    fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<(RequestId, Response<'static>)>> {
        use self::Machine::*;
        match self.state {
            NeedHeader => {
                if buf.len() < Header::encoded_len() {
                    return Ok(None);
                }
                let h = Header::try_from(buf.drain_to(Header::encoded_len())
                        .as_slice()).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
                self.state = WithHeader(h, h.length as usize);

                return self.decode(buf);
            }
            WithHeader(_, l) => {
                if l as usize > buf.len() {
                    return Ok(None);
                }
                use codec::header::OpCode::*;
                use std::mem;
                let h = match mem::replace(&mut self.state, NeedHeader) {
                    WithHeader(h, _) => h,
                    _ => unreachable!(),
                };
                let code = h.op_code.clone();
                let response = Response {
                    store: buf.drain_to(h.length as usize),
                    header: h,
                    message: response::Message::Ready,
                };
                Ok(match code {
                    Supported => {
                        response::SupportedMessage::decode(response.store.as_slice()).map(|res| {
                        /* TODO: verify amount of consumed bytes equals the ones actually parsed */
                                Some((h.stream_id as RequestId,{
                                        response.message =
                                            response::Message::Supported(res.decoded);
                                    response}))
                            })
                            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?
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
