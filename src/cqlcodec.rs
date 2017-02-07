

use codec::request::{RequestBody, OptionsRequest, CqlEncode, Request};
use codec::header::{ProtocolVersion, Direction, Header, OpCode};

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
    type Out = (RequestId, RequestBody);

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

    fn encode(&mut self, msg: (RequestId, RequestBody), buf: &mut Vec<u8>) -> io::Result<()> {
        let (id, msg) = msg;

        let mut body_buf = Vec::new();
        let len = msg.encode(&mut body_buf)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        // if len > u32::max_value() as usize {
        //     return Err(ErrorKind::BodyLengthExceeded(len).into());
        // }

        let len = len as u32;


        let header = Header {
            // version: match msg {
            //     Option => OptionsRequest::protocol_version(),
            //     _ => ,
            // },
            version: ProtocolVersion::Version3(Direction::Request),
            // flags: options.flags,
            flags: 0x00,
            stream_id: id as u16, // quick impl -> TODO: Problem!!
            op_code: match msg {
                Option => OpCode::Options,
            },
            length: len,
        };

        let header_bytes = header.encode()
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

        buf.extend(&header_bytes[..]);
        buf.extend(body_buf);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use codec::request::*;

    #[test]
    fn from_options_request() {
        let o = OptionsRequest;
        let o = RequestBody::Option(o);

        let mut buf = Vec::new();
        // let options = EncodeOptions {
        //     flags: 0,
        //     stream_id: 270,
        // };
        let mut codec = CqlCodecV3;
        codec.encode((270, o), &mut buf).unwrap();


        let expected_bytes = b"\x03\x00\x01\x0e\x05\x00\x00\x00\x00";

        assert_eq!(buf.len(), 9);
        assert_eq!(&buf[..], &expected_bytes[..]);
    }
}
