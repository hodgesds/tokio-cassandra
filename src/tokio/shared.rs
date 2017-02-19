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
        _ => unimplemented!(),
    })
}

pub fn io_err<S>(msg: S) -> io::Error
    where S: Into<Box<::std::error::Error + Send + Sync>>
{
    io::Error::new(io::ErrorKind::Other, msg)
}

use tokio_core::io::{Io, Framed};
use futures::{Sink, Stream, Future};
use super::simple::{Response, CqlCodec};
use codec::request;

fn perform_handshake<T>(transport: Framed<T, CqlCodec>)
                        -> Box<Future<Item = Framed<T, CqlCodec>, Error = io::Error>>
    where T: Io + 'static
{
    Box::new(transport.send((0, request::Message::Options))
        .and_then(|transport| transport.into_future().map_err(|(e, _)| e))
        .and_then(|(res, transport)| {
            res.ok_or_else(|| io_err("No reply received upon 'OPTIONS' message"))
                .and_then(|(_id, res)| match res.message {
                    response::Message::Supported(msg) => {
                        let startup = request::StartupMessage {
                            cql_version: msg.latest_cql_version()
                                .ok_or(io_err("Expected CQL_VERSION to contain at least one \
                                               version"))?
                                .clone(),
                            compression: None,
                        };
                        Ok((transport, startup))
                    }
                    msg => {
                        Err(io_err(format!("Expected to receive 'SUPPORTED' message but got {:?}",
                                           msg)))
                    }
                })
        })
        .and_then(|(transport, startup)| {
            Box::new(transport.send((0, request::Message::Startup(startup)))
                .and_then(|transport| transport.into_future().map_err(|(e, _)| e))
                .and_then(|(res, transport)| {
                    res.ok_or_else(|| io_err("No reply received upon 'STARTUP' message"))
                        .map(|(_id, _res)| transport)
                }))
        }))
}
