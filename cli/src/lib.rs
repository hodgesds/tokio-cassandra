#[macro_use]
extern crate clap;

#[macro_use]
extern crate error_chain;
extern crate tokio_cassandra;
extern crate tokio_core;
extern crate tokio_service;
extern crate futures;
extern crate dns_lookup;
extern crate semver;

pub mod errors {
    use std::num::ParseIntError;
    use std::net::AddrParseError;
    use semver::SemVerError;
    use std::io;

    error_chain!{
        foreign_links {
            ParseInt(ParseIntError);
            AddrParse(AddrParseError);
            SemVerParse(SemVerError);
            Other(io::Error);
        }

       errors {
            Pk12PathFormat(s: String) {
                description("Could not parse pk12 file path description: <path>:<password> is required")
                display("Failed to parse '{}' as <path>:<password>", s)
            }
        }
    }
}

mod args;
mod scmds;

pub use self::scmds::*;
pub use self::args::*;
