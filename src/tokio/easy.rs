use super::streaming::{StreamingMessage, ClientHandle};
use tokio_service::Service;
use futures::Future;
use codec::{response, request};
use std::io;

pub struct EasyClientHandle {
    inner: ClientHandle,
}

impl From<ClientHandle> for EasyClientHandle {
    fn from(f: ClientHandle) -> Self {
        EasyClientHandle { inner: f }
    }
}

#[derive(Debug)]
pub enum Message {
    Supported(response::SupportedMessage),
    Ready,
}

impl From<StreamingMessage> for Message {
    fn from(f: StreamingMessage) -> Self {
        match f {
            StreamingMessage::Ready => Message::Ready,
            StreamingMessage::Supported(msg) => Message::Supported(msg),
            StreamingMessage::Partial(_stream) => {
                // TODO: exhaust stream and build a singular response in a blocking fashion
                unimplemented!()
            }
        }
    }
}

impl Service for EasyClientHandle {
    type Request = request::Message;
    type Response = StreamingMessage;
    type Error = io::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        Box::new(self.inner.call(req).map(From::from))
    }
}
