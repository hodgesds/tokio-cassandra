extern crate clap;

#[macro_use]
extern crate error_chain;
extern crate tokio_cassandra;
extern crate tokio_core;

pub mod errors {
    use std::num::ParseIntError;
    use std::net::AddrParseError;

    error_chain!{
        foreign_links {
            ParseInt(ParseIntError);
            AddrParse(AddrParseError);
        }
    }
}

mod scmds {
    use clap;
    use super::errors::*;
    use tokio_cassandra::tokio::Client;
    use tokio_core::reactor::Core;

    pub fn test_connection(args: &clap::ArgMatches) -> Result<()> {
        let host = args.value_of("host").expect("clap to work");
        let port: u16 = args.value_of("port").expect("clap to work").parse()?;
        let addr = format!("{}:{}", host, port).parse()?;
        let core = Core::new().expect("Core can be created");
        let handle = core.handle();

        let client = Client::connect(&addr, &handle);
        Ok(())
    }

}

pub use self::scmds::*;
//
//impl Client {
//    /// Establish a connection to a multiplexed line-based server at the
//    /// provided `addr`.
//    pub fn connect(addr: &SocketAddr,
//                   handle: &Handle)
//                   -> Box<Future<Item = Client, Error = io::Error>> {
//        let ret = TcpClient::new(LineProto)
//            .connect(addr, handle)
//            .map(|client_service| {
//                let validate = Validate { inner: client_service };
//                Client { inner: validate }
//            });
//
//        Box::new(ret)
//    }
//}
//
//impl Service for Client {
//    type Request = String;
//    type Response = String;
//    type Error = io::Error;
//    // For simplicity, box the future.
//    type Future = Box<Future<Item = String, Error = io::Error>>;
//
//    fn call(&self, req: String) -> Self::Future {
//        self.inner.call(req)
//    }
//}
//
//impl<T> Service for Validate<T>
//    where T: Service<Request = String, Response = String, Error = io::Error>,
//          T::Future: 'static
//{
//    type Request = String;
//    type Response = String;
//    type Error = io::Error;
//    // For simplicity, box the future.
//    type Future = Box<Future<Item = String, Error = io::Error>>;
//
//    fn call(&self, req: String) -> Self::Future {
//        // Make sure that the request does not include any new lines
//        if req.chars().find(|&c| c == '\n').is_some() {
//            let err = io::Error::new(io::ErrorKind::InvalidInput, "message contained new line");
//            return Box::new(future::done(Err(err)));
//        }
//
//        // Call the upstream service and validate the response
//        Box::new(self.inner
//            .call(req)
//            .and_then(|resp| if resp.chars().find(|&c| c == '\n').is_some() {
//                Err(io::Error::new(io::ErrorKind::InvalidInput, "message contained new line"))
//            } else {
//                Ok(resp)
//            }))
//    }
//}
