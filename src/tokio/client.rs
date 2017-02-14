use tokio_service::Service;
use futures::Future;
use codec::{response, request};
use tokio_core::reactor::Handle;
use tokio_proto::TcpClient;
use super::codec::CqlProtoV3;
use std::io;

struct Client;

error_chain!{}

impl Client {
    pub fn connect(host: &str,
                   port: u16,
                   handle: &Handle)
                   -> Box<Future<Item = Client, Error = io::Error>> {
        let addr = host.parse().expect("TODO: dns resolve this one ... in a future :P");
        let ret = TcpClient::new(CqlProtoV3)
            .connect(addr, handle)
            .map(|client_service| Client);
        Box::new(ret)
    }
}

impl Service for Client {
    type Request = request::Message;
    type Response = response::Message;
    type Error = io::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        unimplemented!()
    }
}
