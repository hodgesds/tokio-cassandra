[![Build Status linux+osx](https://travis-ci.org/nhellwig/tokio-cassandra.svg?branch=master)](https://travis-ci.org/nhellwig/tokio-cassandra)
[![crates.io version](https://img.shields.io/crates/v/tokio-cassandra.svg)](https://crates.io/crates/tokio-cassandra)

A Cassandra Native Protocol 3 implementation using Tokio for IO.

# Usage

Add this to your Cargo.toml
```toml
[dependencies]
tokio-cassandra = "*"
```

Add this to your lib ...
```Rust
extern crate tokio_cassandra;
```

# Goals
* implement cassandra v3 protocol leveraging the tokio ecosystem to the fullest. Stream as much as possible to reduce the amount of copies to a minium.
* leave it flexible enough to easily provide support for protocol version 4
