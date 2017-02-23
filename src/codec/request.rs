use codec::header::{ProtocolVersion, OpCode, Header, Version};
use std::collections::HashMap;

use codec::primitives::{CqlStringMap, CqlString, CqlBytes};
use codec::primitives::encode;

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
    fn encode(&self, v: ProtocolVersion, f: &mut Vec<u8>) -> Result<usize>;
}

#[derive(Debug)]
pub enum Message {
    Options,
    Startup(StartupMessage),
    AuthResponse(AuthResponseMessage),
}

use tokio_core::io::EasyBuf;

#[derive(Debug)]
pub struct StartupMessage {
    pub cql_version: CqlString<EasyBuf>,
    pub compression: Option<CqlString<EasyBuf>>,
}

impl CqlEncode for StartupMessage {
    fn encode(&self, _v: ProtocolVersion, buf: &mut Vec<u8>) -> Result<usize> {
        use codec::primitives::CqlFrom;

        let mut sm: HashMap<CqlString<EasyBuf>, CqlString<EasyBuf>> = HashMap::new();
        sm.insert(unsafe { CqlString::unchecked_from("CQL_VERSION") },
                  self.cql_version.clone());

        if let Some(ref c) = self.compression {
            sm.insert(unsafe { CqlString::unchecked_from("COMPRESSION") },
                      c.clone());
        }
        let sm = unsafe { CqlStringMap::unchecked_from(sm) };
        let l = buf.len();
        encode::string_map(&sm, buf);
        Ok(buf.len() - l)
    }
}

#[derive(Debug)]
pub struct AuthResponseMessage {
    pub auth_data: CqlBytes<EasyBuf>,
}

impl CqlEncode for AuthResponseMessage {
    fn encode(&self, _v: ProtocolVersion, buf: &mut Vec<u8>) -> Result<usize> {
        let l = buf.len();
        encode::bytes(&self.auth_data, buf);
        Ok(buf.len() - l)
    }
}

impl Message {
    fn opcode(&self) -> OpCode {
        use self::Message::*;
        match self {
            &Options => OpCode::Options,
            &Startup(_) => OpCode::Startup,
            &AuthResponse(_) => OpCode::AuthResponse,
        }

    }
}


impl CqlEncode for Message {
    fn encode(&self, v: ProtocolVersion, buf: &mut Vec<u8>) -> Result<usize> {

        match *self {
            Message::Options => Ok(0),
            Message::Startup(ref msg) => msg.encode(v, buf),
            Message::AuthResponse(ref msg) => msg.encode(v, buf),
        }
    }
}


pub fn cql_encode(version: ProtocolVersion,
                  flags: u8,
                  stream_id: u16,
                  to_encode: Message,
                  sink: &mut Vec<u8>)
                  -> Result<()> {
    sink.resize(Header::encoded_len(), 0);

    let len = to_encode.encode(version, sink)?;
    if len > u32::max_value() as usize {
        return Err(ErrorKind::BodyLengthExceeded(len).into());
    }
    let len = len as u32;

    let header = Header {
        version: Version::request(version),
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
    use codec::header::ProtocolVersion::*;
    use codec::primitives::CqlFrom;
    use codec::authentication::Authenticator;

    #[test]
    fn from_options_request() {
        let o = Message::Options;

        let mut buf = Vec::new();
        let flags = 0;
        let stream_id = 270;
        cql_encode(Version3, flags, stream_id, o, &mut buf).unwrap();

        let expected_bytes = b"\x03\x00\x01\x0e\x05\x00\x00\x00\x00";

        assert_eq!(&buf[..], &expected_bytes[..]);
    }

    #[test]
    fn from_startup_req() {
        let o = Message::Startup(StartupMessage {
            cql_version: CqlString::try_from("3.2.1").unwrap(),
            compression: None,
        });

        let mut buf = Vec::new();
        let flags = 0;
        let stream_id = 1;
        cql_encode(Version3, flags, stream_id, o, &mut buf).unwrap();

        let expected_bytes = include_bytes!("../../tests/fixtures/v3/requests/cli_startup.msg");

        assert_eq!(&buf[..], &expected_bytes[..]);
    }

    #[test]
    fn from_auth_response_req() {
        let a = Authenticator::PlainTextAuthenticator {
            username: String::from("abcdef12"),
            password: String::from("123456789asdfghjklqwertyuiopzx"),
        };

        let mut v = Vec::new();
        a.encode_auth_response(&mut v);

        println!("v.len() = {:?}", v.len());

        let o = Message::AuthResponse(AuthResponseMessage {
            auth_data: CqlBytes::try_from(v).unwrap(),
        });

        let mut buf = Vec::new();
        let flags = 0;
        let stream_id = 2;
        cql_encode(Version3, flags, stream_id, o, &mut buf).unwrap();

        let expected_bytes = include_bytes!("../../tests/fixtures/v3/requests/auth_response.msg");

        assert_eq!(&buf[..], &expected_bytes[..]);
    }
}
