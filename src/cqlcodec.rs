

use codec::request::{Request, cql_encode};

use tokio_proto::streaming::multiplex::RequestId;
use tokio_core::io::{EasyBuf, Codec};
use std::io;

error_chain! {
    foreign_links {
        Io(::std::io::Error);
    }
}

struct CqlCodecV3;
impl Codec for CqlCodecV3 {
    type In = (RequestId, String);
    type Out = (RequestId, Request);

    fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<(RequestId, String)>> {
        // // At least 5 bytes are required for a frame: 4 byte
        // // head + one byte '\n'
        // if buf.len() < 5 {
        //     // We don't yet have a full message
        //     return Ok(None);
        // }

        // // Check to see if the frame contains a new line, skipping
        // // the first 4 bytes which is the request ID
        // let newline = buf.as_ref()[4..].iter().position(|b| *b == b'\n');
        // if let Some(n) = newline {
        //     // remove the serialized frame from the buffer.
        //     let line = buf.drain_to(n + 4);

        //     // Also remove the '\n'
        //     buf.drain_to(1);

        //     // Deserialize the request ID
        //     let id = BigEndian::read_u32(&line.as_ref()[0..4]);

        //     // Turn this data into a UTF string and return it in a Frame.
        //     return match str::from_utf8(&line.as_ref()[4..]) {
        //         Ok(s) => Ok(Some((id as RequestId, s.to_string()))),
        //         Err(_) => Err(io::Error::new(io::ErrorKind::Other,
        //                                      "invalid string")),
        //     }
        // }

        // No `\n` found, so we don't have a complete message
        Ok(None)
    }

    fn encode(&mut self, msg: (RequestId, Request), buf: &mut Vec<u8>) -> io::Result<()> {
        let (id, req) = msg;

        cql_encode(0x00, /* FIXME real flags */
                   id as u16, /* FIXME safe cast */
                   req,
                   buf)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
    }
}

//#[cfg(test)]
//mod test {
//    #[test]
//    fn decode_supported() {
//        let frame = include_bytes!("../tests/fixtures/v3/srv_supported.msg");
//        //        let r = Response::decode(&frame);
//    }
//}
