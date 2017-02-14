extern crate tcc as line;

extern crate futures;
extern crate tokio_core;
extern crate tokio_service;

use futures::Future;
use tokio_core::reactor::Core;
use tokio_service::Service;

pub fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let addr = "127.0.0.1".parse().unwrap();
    core.run(
        line::Client::connect(&addr, &handle)
            .and_then(|client| {
                client.call("Hello".to_string())
                    .and_then(move |response| {
                        println!("CLIENT: {:?}", response);
                        client.call("Goodbye".to_string())
                    })
                    .and_then(|response| {
                        println!("CLIENT: {:?}", response);
                        Ok(())
                    })
            })
    ).unwrap();
}
