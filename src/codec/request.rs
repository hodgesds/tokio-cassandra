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

pub trait CqlEncode {
    fn encode(&self, f: &mut Vec<u8>) -> Result<usize>;
}

pub trait Request {
    fn opcode() -> OpCode;
    fn protocol_version() -> ProtocolVersion;
}

pub struct OptionsRequest;

impl Request for OptionsRequest {
    fn opcode() -> OpCode {
        OpCode::Options
    }

    fn protocol_version() -> ProtocolVersion {
        ProtocolVersion::Version3(Direction::Request)
    }
}

impl CqlEncode for OptionsRequest {
    fn encode(&self, _: &mut Vec<u8>) -> Result<usize> {
        Ok(0)
    }
}

pub struct EncodeOptions {
    pub flags: u8,
    pub stream_id: u16,
}

pub fn cql_encode<E>(options: EncodeOptions,
                     to_encode: E,
                     body_buf: &mut Vec<u8>)
                     -> Result<([u8; 9])>
    where E: Request + CqlEncode
{
    let len = to_encode.encode(body_buf)?;
    if len > u32::max_value() as usize {
        return Err(ErrorKind::BodyLengthExceeded(len).into());
    }
    let len = len as u32;

    let header = Header {
        version: E::protocol_version(),
        flags: options.flags,
        stream_id: options.stream_id,
        op_code: E::opcode(),
        length: len,
    };

    let header_bytes = header.encode()?;
    Ok(header_bytes)
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_options_request() {
        let o = OptionsRequest;
        let mut buf = Vec::new();
        let options = EncodeOptions {
            flags: 0,
            stream_id: 270,
        };
        let header_bytes = cql_encode(options, o, &mut buf).unwrap();

        let expected_bytes = b"\x03\x00\x01\x0e\x05\x00\x00\x00\x00";

        assert_eq!(buf.len(), 0);
        assert_eq!(&header_bytes[..], &expected_bytes[..]);
    }
}
