use std::io;
use codec::header::{OpCode, Header, ProtocolVersion, Direction};

error_chain! {
    foreign_links {
        Io(::std::io::Error);
        HeaderError(::codec::header::Error);
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
    pub fn encode<E, W>(&self, to_encode: E, to_write_to: &mut W) -> Result<usize>
        where E: CqlProtoEncode<Vec<u8>>,
              W: io::Write
    {
        let mut body: Vec<u8> = Vec::new();
        to_encode.encode(&mut body)?;

        let header = Header {
            version: E::protocol_version(),
            flags: 0x00,
            stream_id: 270,
            op_code: E::opcode(),
            length: body.len() as u32, // PROBLEM?
        };

        header.encode(to_write_to)?;
        to_write_to.write(&body)?;

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
        let len = e.encode(o, &mut buf).unwrap();

        let expected_bytes = b"\x03\x00\x01\x0e\x05\x00\x00\x00\x00";

        assert_eq!(len, 9);
        assert_eq!(&buf[..], &expected_bytes[..]);
    }

}
