/*
  The CQL binary protocol is a frame based protocol. Frames are defined as:

      0         8        16        24        32         40
      +---------+---------+---------+---------+---------+
      | version |  flags  |      stream       | opcode  |
      +---------+---------+---------+---------+---------+
      |                length                 |
      +---------+---------+---------+---------+
      |                                       |
      .            ...  body ...              .
      .                                       .
      .                                       .
      +----------------------------------------

  The protocol is big-endian (network byte order).

  Each frame contains a fixed size header (9 bytes) followed by a variable size
  body. The header is described in Section 2. The content of the body depends
  on the header opcode value (the body can in particular be empty for some
  opcode values). The list of allowed opcode is defined Section 2.4 and the
  details of each corresponding message is described Section 4.

  The protocol distinguishes 2 types of frames: requests and responses. Requests
  are those frame sent by the clients to the server, response are the ones sent
  by the server. Note however that the protocol supports server pushes (events)
  so responses does not necessarily come right after a client request.

  Note to client implementors: clients library should always assume that the
  body of a given frame may contain more data than what is described in this
  document. It will however always be safe to ignore the remaining of the frame
  body in such cases. The reason is that this may allow to sometimes extend the
  protocol with optional features without needing to change the protocol
  version.
**/

use byteorder::{BigEndian, ReadBytesExt};
const HEADER_LENGTH: usize = 9;

error_chain! {
    errors {
        UnsupportedVersion(v: u8) {
            description("The protocol version of cassandra is unsupported")
            display("The protocol version {} of cassandra is unsupported", v)
        }
        InvalidDataLength(l: usize) {
            description("The size of data is incorrect")
            display("Expected a data size of {} bytes, but got {}", HEADER_LENGTH, l)
        }
        InvalidOpCode(c: u8) {
            description("Could not parse value as opcode.")
            display("Value {} is not a valid opcode", c)
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum OpCode {
    Error,
    Startup,
    Ready,
    Authenticate,
    Options,
    Supported,
    Query,
    Result,
    Prepare,
    Execute,
    Register,
    Event,
    Batch,
    AuthChallenge,
    AuthResponse,
    AuthSuccess,
}


impl OpCode {
    pub fn try_from(b: u8) -> Result<OpCode> {
        Ok(match b {
            0x00 => OpCode::Error,
            0x01 => OpCode::Startup,
            0x02 => OpCode::Ready,
            0x03 => OpCode::Authenticate,
            0x05 => OpCode::Options,
            0x06 => OpCode::Supported,
            0x07 => OpCode::Query,
            0x08 => OpCode::Result,
            0x09 => OpCode::Prepare,
            0x0A => OpCode::Execute,
            0x0B => OpCode::Register,
            0x0C => OpCode::Event,
            0x0D => OpCode::Batch,
            0x0E => OpCode::AuthChallenge,
            0x0F => OpCode::AuthResponse,
            0x10 => OpCode::AuthSuccess,
            _ => return Err(ErrorKind::InvalidOpCode(b).into()),
        })
    }
}



#[derive(PartialEq, Debug, Clone)]
pub struct Header {
    pub version: ProtocolVersion,
    pub flags: u8,
    pub stream_id: u16,
    pub op_code: OpCode,
    pub length: u32,
}

impl Header {
    pub fn try_from(b: &[u8]) -> Result<Header> {
        if b.len() < HEADER_LENGTH {
            return Err(ErrorKind::InvalidDataLength(b.len()).into());
        }

        let version = match b[0] {
            0x03 => ProtocolVersion::Version3,
            _ => return Err(ErrorKind::UnsupportedVersion(b[0]).into()),
        };

        Ok(Header {
            version: version,
            flags: b[1],
            stream_id: (&b[2..4]).read_u16::<BigEndian>().expect("to have 2 bytes exactly"),
            op_code: OpCode::try_from(b[4])?,
            length: (&b[5..9]).read_u32::<BigEndian>().expect("to have 4 bytes exactly"),
        })
    }

    pub fn is_compressed(&self) -> bool {
        self.flags & 0x01 == 0x01
    }

    pub fn is_traced(&self) -> bool {
        self.flags & 0x02 == 0x02
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum ProtocolVersion {
    Version3,
}

#[cfg(test)]
mod test {
    use std::ops::Deref;
    use std::fmt::{Display, Debug};
    use std::result::Result as StdResult;
    use error_chain::ChainedError;
    use super::*;

    #[test]
    fn test_empty_frame() {
        let bytes = b"\x03\x00\x01\x01\x05\x00\x00\x01\x05";
        let h = Header::try_from(&bytes[..]).unwrap();

        assert_eq!(h.version, ProtocolVersion::Version3);
        assert_eq!(h.is_compressed(), false);
        assert_eq!(h.is_traced(), false);
        assert_eq!(h.stream_id, 257);
        assert_eq!(h.op_code, OpCode::Options);
        assert_eq!(h.length, 261);
    }

    #[test]
    fn test_invalid_op_code() {
        let bytes = b"\x03\x00\x01\x01\x04\x00\x00\x00\x00";
        let res = Header::try_from(&bytes[..]);

        assert!(err_is(res, ErrorKind::InvalidOpCode(0x04)));
    }
    #[test]
    fn test_invalid_length_of_data() {
        let bytes = b"\x03\x00\x01\x01\x05\x00\x00\x00";
        let res = Header::try_from(&bytes[..]);

        assert!(err_is(res, ErrorKind::InvalidDataLength(8)));
    }

    #[test]
    fn test_empty_frame_compressed() {
        let bytes = b"\x03\x01\x00\x00\x05\x00\x00\x00\x00";
        let h = Header::try_from(&bytes[..]).unwrap();

        assert_eq!(h.is_compressed(), true);
        assert_eq!(h.is_traced(), false);
    }

    #[test]
    fn test_empty_frame_traced() {
        let bytes = b"\x03\x02\x00\x00\x05\x00\x00\x00\x00";
        let h = Header::try_from(&bytes[..]).unwrap();

        assert_eq!(h.is_compressed(), false);
        assert_eq!(h.is_traced(), true);
    }

    #[test]
    fn test_unsupported_version() {
        let bytes = b"\x04\x02\x00\x00\x05\x00\x00\x00\x00";
        let res = Header::try_from(&bytes[..]);

        assert!(err_is(res, ErrorKind::UnsupportedVersion(0x04)));
    }


    fn err_is<T, E>(res: StdResult<T, E>, kind: E::Target) -> bool
        where E: Deref + ChainedError + Debug + Display,
              E::Target: Display + Sized,
              E::ErrorKind: Display,
              T: Debug
    {
        format!("{}", res.unwrap_err().kind()) == format!("{}", kind)
    }
}
