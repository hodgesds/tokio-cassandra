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
        client.wait().unwrap();
        Ok(())
    }

}

pub use self::scmds::*;
