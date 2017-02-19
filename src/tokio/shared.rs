//! Code shared by streaming and simple protocol versions
use tokio_core::io::EasyBuf;
use codec::header::{OpCode, ProtocolVersion};
use codec::response::{self, CqlDecode};

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
        _ => unimplemented!(),
    })
}
