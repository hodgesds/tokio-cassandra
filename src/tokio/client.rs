use super::CqlProtoV3;
use codec::{request, response};
use tokio_service::Service;
use futures::Future;
use tokio_core::reactor::Handle;
use tokio_proto::TcpClient;
use std::io;

pub struct Client;

impl Client {
    pub fn connect(host: &str,
                   _port: u16,
                   handle: &Handle)
                   -> Box<Future<Item = Client, Error = io::Error>> {
        let addr = host.parse().expect("TODO: dns resolve this one ... in a future :P");
        let ret = TcpClient::new(CqlProtoV3)
            .connect(&addr, handle)
            .map(|_client_service| Client);
        Box::new(ret)
    }
}

impl Service for Client {
    type Request = request::Message;
    type Response = response::Message;
    type Error = io::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, _req: Self::Request) -> Self::Future {
        unimplemented!()
    }
}
