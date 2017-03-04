use std::io;
use std::sync::Arc;
use std::net::SocketAddr;
use std::marker::PhantomData;

use tokio_proto::BindClient;
use tokio_core::reactor::Handle;
use tokio_core::net::TcpStream;
use tokio_openssl::{SslStream, SslConnectorExt};
use futures::{Future, Poll, Async};
use openssl::ssl::{SslMethod, SslConnectorBuilder};

use super::Options;

pub struct SslClient<Kind, P> {
    _kind: PhantomData<Kind>,
    proto: Arc<P>,
    _tls: Options,
}

pub struct Connect<Kind, P> {
    _kind: PhantomData<Kind>,
    proto: Arc<P>,
    socket: Box<Future<Item = SslStream<TcpStream>, Error = io::Error>>,
    handle: Handle,
}

impl<Kind, P> Future for Connect<Kind, P>
    where P: BindClient<Kind, SslStream<TcpStream>>
{
    type Item = P::BindClient;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<P::BindClient, io::Error> {
        let socket = try_ready!(self.socket.poll().map_err(|e| io::Error::new(io::ErrorKind::Other, e)));
        Ok(Async::Ready(self.proto.bind_client(&self.handle, socket)))
    }
}

impl<Kind, P> SslClient<Kind, P>
    where P: BindClient<Kind, SslStream<TcpStream>>
{
    pub fn new(protocol: P, _tls: Options) -> SslClient<Kind, P> {
        SslClient {
            _kind: PhantomData,
            proto: Arc::new(protocol),
            _tls: _tls,
        }
    }

    pub fn connect(&self, addr: &SocketAddr, handle: &Handle) -> Connect<Kind, P> {
        Connect {
            _kind: PhantomData,
            proto: self.proto.clone(),
            socket: {
                Box::new(TcpStream::connect(addr, handle).and_then(|stream| {
                    let connector = SslConnectorBuilder::new(SslMethod::tls()).unwrap().build();
                    connector.connect_async("google", stream)
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
                }))
            },
            handle: handle.clone(),
        }
    }
}
