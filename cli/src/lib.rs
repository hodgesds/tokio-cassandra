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

mod args;
mod scmds;

pub use self::scmds::*;
pub use self::args::*;
