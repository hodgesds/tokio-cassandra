extern crate clap;

#[macro_use]
extern crate error_chain;
extern crate tokio_cassandra;
extern crate tokio_core;

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
    use tokio_cassandra::tokio::Client;
    use tokio_core::reactor::Core;

    pub fn test_connection(args: &clap::ArgMatches) -> Result<()> {
        let host = args.value_of("host").expect("clap to work");
        let port = args.value_of("port").expect("clap to work");
        let port: u16 = port.parse()
            .chain_err(|| format!("Port '{}' could not be parsed as number", port))?;
        let addr = format!("{}:{}", host, port).parse()
            .chain_err(|| format!("Host '{}' could not be parsed as IP", host))?;

        let mut core = Core::new().expect("Core can be created");
        let handle = core.handle();

        let client = Client::connect(&addr, &handle);
        core.run(client)
            .chain_err(|| format!("Failed to connect to {}:{}", host, port))
            .map(|_| ())
            .map_err(|e| e.into())
    }

}

pub use self::scmds::*;
