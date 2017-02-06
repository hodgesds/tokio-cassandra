use std::io;
use codec::header::{OpCode, Header, ProtocolVersion, Direction};

error_chain! {
    foreign_links {
        Io(::std::io::Error);
        HeaderError(::codec::header::Error);
    }
    errors {
        BodyLengthExceeded(len: usize) {
            description("The length of the body exceeded the \
            maximum length specified by the protocol")
            display("The current body length {} exceeded the \
            maximum allowed length for a body", len)
        }
    }
}

pub trait CqlProtoEncode<W> {
    fn opcode() -> OpCode;
    fn protocol_version() -> ProtocolVersion;
    fn encode(&self, f: &mut W) -> Result<usize>;
}

pub struct OptionsRequest;
impl<W> CqlProtoEncode<W> for OptionsRequest
    where W: io::Write
{
    fn opcode() -> OpCode {
        OpCode::Options
    }

    fn protocol_version() -> ProtocolVersion {
        ProtocolVersion::Version3(Direction::Request)
    }

    fn encode(&self, _: &mut W) -> Result<usize> {
        Ok(0)
    }
}

pub struct CqlEncoder;

impl CqlEncoder {
    pub fn encode<E, W>(&self,
                        to_encode: E,
                        to_write_to: &mut W,
                        body_buf: &mut Vec<u8>)
                        -> Result<usize>
        where E: CqlProtoEncode<Vec<u8>>,
              W: io::Write
    {
        assert!(body_buf.len() == 0,
                "Can't handle non-empty body-bufs for now");
        let len = to_encode.encode(body_buf)?;
        if len > u32::max_value() as usize {
            return Err(ErrorKind::BodyLengthExceeded(len).into());
        }
        let len = len as u32;

        let header = Header {
            version: E::protocol_version(),
            flags: 0x00,
            stream_id: 270,
            op_code: E::opcode(),
            length: len,
        };

        header.encode(to_write_to)?;
        to_write_to.write(&body_buf.as_ref())?;

        Ok(9)
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_options_request() {
        let o = OptionsRequest;
        let e = CqlEncoder;
        let mut buf = Vec::new();
        let mut body_buf = Vec::new();
        let len = e.encode(o, &mut buf, &mut body_buf).unwrap();

        let expected_bytes = b"\x03\x00\x01\x0e\x05\x00\x00\x00\x00";

        assert_eq!(len, 9);
        assert_eq!(&buf[..], &expected_bytes[..]);
    }

}
