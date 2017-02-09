use codec::header::{OpCode, Header, ProtocolVersion, Direction};

use codec::primitives::CqlString;

error_chain! {
    foreign_links {
        Io(::std::io::Error);
        HeaderError(::codec::header::Error);
        PrimitiveError(::codec::primitives::Error);
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

pub enum Message<'a> {
    Options,
    Startup(StartupMessage<'a>),
}

pub struct StartupMessage<'a> {
    _cql_version: CqlString<'a>,
    _compression: Option<CqlString<'a>>,
}


impl<'a> Message<'a> {
    fn opcode(&self) -> OpCode {
        use self::Message::*;
        match self {
            &Options => OpCode::Options,
            &Startup(_) => OpCode::Startup,
        }

    }
    fn protocol_version() -> ProtocolVersion {
        ProtocolVersion::Version3(Direction::Request)
    }
}


impl<'a> CqlEncode for Message<'a> {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<usize> {
        use codec::primitives::encode;
        use codec::primitives::CqlStringMap;

        match *self {
            Message::Options => Ok(0),
            Message::Startup(_) => {
                let sm = CqlStringMap::try_from_iter(vec![(CqlString::try_from("CQL_VERSION")?,
                                                           CqlString::try_from("3.2.1")?)])?;
                //                data.cql_version)])?;
                let l = buf.len();
                encode::string_map(&sm, buf);
                Ok(buf.len() - l)
            }
        }
    }
}


pub fn cql_encode(flags: u8, stream_id: u16, to_encode: Message, sink: &mut Vec<u8>) -> Result<()> {
    sink.resize(Header::encoded_len(), 0);

    let len = to_encode.encode(sink)?;
    if len > u32::max_value() as usize {
        return Err(ErrorKind::BodyLengthExceeded(len).into());
    }
    let len = len as u32;

    let header = Header {
        version: Message::protocol_version(),
        flags: flags,
        stream_id: stream_id,
        op_code: to_encode.opcode(),
        length: len,
    };

    let header_bytes = header.encode()?;
    sink[0..Header::encoded_len()].copy_from_slice(&header_bytes);

    Ok(())
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_options_request() {
        let o = Message::Options;

        let mut buf = Vec::new();
        let flags = 0;
        let stream_id = 270;
        cql_encode(flags, stream_id, o, &mut buf).unwrap();

        let expected_bytes = b"\x03\x00\x01\x0e\x05\x00\x00\x00\x00";

        assert_eq!(&buf[..], &expected_bytes[..]);
    }

    #[test]
    fn from_startup_request() {
        let m = StartupMessage {
            _cql_version: CqlString::try_from("3.2.1").unwrap(),
            _compression: None,
        };
        let o = Message::Startup(m);

        let mut buf = Vec::new();
        let flags = 0;
        let stream_id = 1;
        cql_encode(flags, stream_id, o, &mut buf).unwrap();

        let expected_bytes = include_bytes!("../../tests/fixtures/v3/requests/cli_startup.msg");

        assert_eq!(&buf[..], &expected_bytes[..]);
    }
}
