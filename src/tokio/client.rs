use super::{Response, CqlProtoV3};
use codec::request;
use tokio_service::Service;
use futures::Future;
use tokio_core::reactor::Handle;
use tokio_proto::{multiplex, TcpClient};
use tokio_core::net::TcpStream;
use std::io;
use std::net::SocketAddr;

pub struct Client {
    inner: multiplex::ClientService<TcpStream, CqlProtoV3>,
}

impl Client {
    pub fn connect(addr: &SocketAddr,
                   handle: &Handle)
                   -> Box<Future<Item = Client, Error = io::Error>> {
        let ret = TcpClient::new(CqlProtoV3)
            .connect(addr, handle)
            .map(|_client_service| Client { inner: _client_service });
        Box::new(ret)
    }
}

impl Service for Client {
    type Request = request::Message;
    type Response = Response;
    type Error = io::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        self.inner.call(req).boxed()
    }
}
