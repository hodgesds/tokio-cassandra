//! Code shared by streaming and simple protocol versions
use tokio_core::io::EasyBuf;
use codec::header::{OpCode, ProtocolVersion};
use codec::response::{self, CqlDecode};
use tokio_core::io::{Io, Framed, Codec};
use tokio_proto::multiplex::RequestId;
use futures::{Sink, Stream, Future};
use codec::request;
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

pub struct SimpleResponse(pub RequestId, pub response::Message);
pub struct SimpleRequest(pub RequestId, pub request::Message);

pub fn perform_handshake<T, C>(transport: Framed<T, C>)
                               -> Box<Future<Item = Framed<T, C>, Error = io::Error>>
    where T: Io + 'static,
          C: Codec + 'static,
          SimpleResponse: From<C::In>,
          C::Out: From<SimpleRequest>
{
    Box::new(transport.send(SimpleRequest(0, request::Message::Options).into())
        .and_then(|transport| transport.into_future().map_err(|(e, _)| e))
        .and_then(|(res, transport)| {
            res.ok_or_else(|| io_err("No reply received upon 'OPTIONS' message"))
                .and_then(|response| {
                    let SimpleResponse(_id, res) = response.into();
                    match res {
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
                            Err(io_err(format!("Expected to receive 'SUPPORTED' message but got \
                                                {:?}",
                                               msg)))
                        }
                    }
                })
        })
        .and_then(|(transport, startup)| {
            Box::new(transport.send(SimpleRequest(0, request::Message::Startup(startup)).into())
                .and_then(|transport| transport.into_future().map_err(|(e, _)| e))
                .and_then(|(res, transport)| {
                    res.ok_or_else(|| io_err("No reply received upon 'STARTUP' message"))
                        .map(|_response| {
                            // TODO: verify we actually got a startup response!
                            transport
                        })
                }))
        }))
}
