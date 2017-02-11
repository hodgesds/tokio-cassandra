#[macro_use]
extern crate error_chain;
extern crate byteorder;

#[cfg(feature = "with-serde")]
extern crate serde;

#[cfg(feature = "with-serde")]
#[macro_use]
extern crate serde_derive;

extern crate tokio_core;
extern crate tokio_proto;

pub mod codec;
pub mod adapter;
