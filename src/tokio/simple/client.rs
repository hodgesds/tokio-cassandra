use super::{Response, CqlProto};
use codec::request;
use tokio_service::Service;
use futures::Future;
use tokio_core::reactor::Handle;
use tokio_proto::{multiplex, TcpClient};
use tokio_core::net::TcpStream;
use std::io;
use std::net::SocketAddr;

pub struct ClientHandle {
    inner: multiplex::ClientService<TcpStream, CqlProto>,
}

pub struct Client {
    pub protocol: CqlProto,
}

impl Client {
    pub fn connect(self,
                   addr: &SocketAddr,
                   handle: &Handle)
                   -> Box<Future<Item = ClientHandle, Error = io::Error>> {
        let ret = TcpClient::new(self.protocol)
            .connect(addr, handle)
            .map(|client_service| ClientHandle { inner: client_service });
        Box::new(ret)
    }
}

impl Service for ClientHandle {
    type Request = request::Message;
    type Response = Response;
    type Error = io::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        self.inner.call(req).boxed()
    }
}
