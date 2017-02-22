//! Code shared by streaming and simple protocol versions
use tokio_core::io::EasyBuf;
use codec::header::{OpCode, ProtocolVersion};
use codec::response::{self, CqlDecode};
use std::io;

pub fn decode_complete_message_by_opcode(version: ProtocolVersion,
                                         code: OpCode,
                                         buf: EasyBuf)
                                         -> response::Result<response::Message> {
    use codec::header::OpCode::*;
    Ok(match code {
        Supported => {
            response::Message::Supported(response::SupportedMessage::decode(version, buf)?)
        }
        Ready => response::Message::Ready,
        Authenticate => {
            response::Message::Authenticate(response::AuthenticateMessage::decode(version, buf)?)
        }
        AuthSuccess => {
            response::Message::AuthSuccess(response::AuthSuccessMessage::decode(version, buf)?)
        }
        Error => response::Message::Error(response::ErrorMessage::decode(version, buf)?),
        _ => unimplemented!(),
    })
}

pub fn io_err<S>(msg: S) -> io::Error
    where S: Into<Box<::std::error::Error + Send + Sync>>
{
    io::Error::new(io::ErrorKind::Other, msg)
}
