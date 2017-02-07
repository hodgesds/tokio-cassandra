

use codec::request::{self, cql_encode};
use codec::header::Header;

use tokio_proto::streaming::multiplex::RequestId;
use tokio_core::io::{EasyBuf, Codec};
use std::io;

pub struct CqlCodecV3;
impl Codec for CqlCodecV3 {
    type In = (RequestId, String);
    type Out = (RequestId, request::Message);

    fn decode(&mut self, _buf: &mut EasyBuf) -> io::Result<Option<(RequestId, String)>> {
        if _buf.len() < Header::encoded_len() {
            return Ok(None);
        }

        // TODO: implement body parsing

        Ok(None)
    }

    fn encode(&mut self, msg: (RequestId, request::Message), buf: &mut Vec<u8>) -> io::Result<()> {
        let (id, req) = msg;

        cql_encode(0x00, /* FIXME real flags */
                   id as u16, /* FIXME safe cast */
                   req,
                   buf)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
    }
}
