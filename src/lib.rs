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

extern crate futures;
extern crate tokio_core;
extern crate tokio_service;
extern crate tokio_proto;

pub mod codec;
pub mod tokio;
