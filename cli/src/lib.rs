extern crate clap;

#[macro_use]
extern crate error_chain;
extern crate tokio_cassandra;
extern crate tokio_core;
extern crate tokio_service;
extern crate futures;


pub mod errors {
    use std::num::ParseIntError;
    use std::net::AddrParseError;
    use std::io;

    error_chain!{
        foreign_links {
            ParseInt(ParseIntError);
            AddrParse(AddrParseError);
            Other(io::Error);
        }
    }
}

mod scmds {
    use clap;
    use super::errors::*;
    use futures::Future;
    use tokio_cassandra::blocking::{CqlProto, Client};
    use tokio_cassandra::codec::request;
    use tokio_cassandra::codec::header::ProtocolVersion;
    use tokio_core::reactor::Core;
    use tokio_service::Service;

    pub fn test_connection(args: &clap::ArgMatches) -> Result<()> {
        let host = args.value_of("host").expect("clap to work");
        let port = args.value_of("port").expect("clap to work");
        let port: u16 = port.parse()
            .chain_err(|| format!("Port '{}' could not be parsed as number", port))?;
        let addr = format!("{}:{}", host, port).parse()
            .chain_err(|| format!("Host '{}' could not be parsed as IP", host))?;

        let mut core = Core::new().expect("Core can be created");
        let handle = core.handle();

        let client = Client { protocol: CqlProto { version: ProtocolVersion::Version3 } }
            .connect(&addr, &handle)
            .and_then(|client| client.call(request::Message::Options));
        core.run(client)
            .chain_err(|| format!("Failed to connect to {}:{}", host, port))
            .map(|_response| {
                println!("Connection to {}:{} successful", host, port);
                ()
            })
            .map_err(|e| e.into())
    }

}

pub use self::scmds::*;
