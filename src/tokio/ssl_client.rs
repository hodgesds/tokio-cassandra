use std::io;
use std::sync::Arc;
use std::net::SocketAddr;
use std::marker::PhantomData;

use tokio_proto::BindClient;
use tokio_core::reactor::Handle;
use tokio_core::net::{TcpStream, TcpStreamNew};
use futures::{Future, Poll, Async};

pub struct TcpClient<Kind, P> {
    _kind: PhantomData<Kind>,
    proto: Arc<P>,
}

pub struct Connect<Kind, P> {
    _kind: PhantomData<Kind>,
    proto: Arc<P>,
    socket: TcpStreamNew,
    handle: Handle,
}

impl<Kind, P> Future for Connect<Kind, P>
    where P: BindClient<Kind, TcpStream>
{
    type Item = P::BindClient;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<P::BindClient, io::Error> {
        let socket = try_ready!(self.socket.poll());
        Ok(Async::Ready(self.proto.bind_client(&self.handle, socket)))
    }
}

impl<Kind, P> TcpClient<Kind, P>
    where P: BindClient<Kind, TcpStream>
{
    pub fn new(protocol: P) -> TcpClient<Kind, P> {
        TcpClient {
            _kind: PhantomData,
            proto: Arc::new(protocol),
        }
    }

    pub fn connect(&self, addr: &SocketAddr, handle: &Handle) -> Connect<Kind, P> {
        Connect {
            _kind: PhantomData,
            proto: self.proto.clone(),
            socket: TcpStream::connect(addr, handle),
            handle: handle.clone(),
        }
    }
}
