#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate quick_error;
extern crate byteorder;

#[cfg(feature = "with-serde")]
extern crate serde;

#[cfg(feature = "with-serde")]
#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate futures;
extern crate tokio_core;
extern crate tokio_service;
extern crate tokio_proto;
#[cfg(feature = "ssl")]
extern crate tokio_openssl;
#[cfg(feature = "ssl")]
extern crate openssl;

extern crate semver;

#[macro_use]
extern crate log;

#[macro_use]
mod macros;

pub mod codec;
mod tokio;
pub use self::tokio::*;
