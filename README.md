<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->
A Cassandra Native Protocol 3 implementation using Tokio for IO.

- [Usage](#usage)
- [Goals](#goals)
  - [General](#general)
  - [Minimal Viable Product and v1.0](#minimal-viable-product-and-v10)
- [Status](#status)
  - [Commandline Interface](#commandline-interface)
  - [Library](#library)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

[![Build Status linux+osx](https://travis-ci.org/nhellwig/tokio-cassandra.svg?branch=master)](https://travis-ci.org/nhellwig/tokio-cassandra)
[![crates.io version](https://img.shields.io/crates/v/tokio-cassandra.svg)](https://crates.io/crates/tokio-cassandra)

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
## General
* implement cassandra v3 protocol leveraging the tokio ecosystem to the fullest.
* safety first - the client will verify all input received from the server.
* test-first development - no code exists unless a test needs it to pass.
* high-performance - stream as much as possible and reduce amount of allocations to a minimum.
* leave it flexible enough to easily provide support for protocol version 4 and later 5.
* develop breadth first - thus we are implementing orthogonal features first to learn how that affects the API and architecture, before implementing every single data-type or message-type.
* strive for an MVP and version 1.0 fast, even if that includes only the most common usecases.
* Prefer to increment major version rapidly instead of keeping major version zero for longer than needed.

## Minimal Viable Product and v1.0
* library supports basic queries without UDTs and can provide the result via an unauthenticated and unencrypted connection.
* a CLI allows to perform such a query, and output results in JSON format.

# Status
## [Commandline Interface](https://github.com/nhellwig/tokio-cassandra/projects/1)
* **test-connection**
  * [x] unauthenticated
  * [ ] authenticated
  * [ ] with-TLS
  * [ ] choice of cql version to use
  * [ ] choice of which protocol version to use
  * [ ] host-name resolution
  * [x] use latest-supported cql version

## [Library](https://github.com/nhellwig/tokio-cassandra/projects/2)
* **Architecture and API**
  * [ ] [multi-protocol-version support](https://github.com/nhellwig/tokio-cassandra/issues/4)
* **Protocol Versions**
  * [ ] v3
  * [ ] v4
* **Transport**
  * **Multiplexed**
    * [x] non-streaming
    * [ ] [streaming](https://github.com/nhellwig/tokio-cassandra/issues/3)
  * [x] unencrypted
  * [ ] [encryption via TLS](https://github.com/nhellwig/tokio-cassandra/issues/5)
* **Connection**
  * [x] unauthenticated
  * [ ] [authenticated](https://github.com/nhellwig/tokio-cassandra/issues/7)
* **Codec V3**
  * [x] frame-header
  * **Message Data Types (MDT)**
    * [x] int
    * [x] long
    * [x] short
    * [x] string
    * [x] long string
    * [ ] [uuid](https://github.com/nhellwig/tokio-cassandra/projects/2#card-1774756)
    * [ ] [option](https://github.com/nhellwig/tokio-cassandra/projects/2#card-1774765)
    * [ ] [option list](https://github.com/nhellwig/tokio-cassandra/projects/2#card-1774766)
    * [ ] [inet](https://github.com/nhellwig/tokio-cassandra/projects/2#card-1774767)
    * [ ] [consistency](https://github.com/nhellwig/tokio-cassandra/projects/2#card-1774768)
    * [x] string map
    * [x] string multi-map
  * **Messages**
    * [ ] Paging
    * **Compression**
      * [ ] Snappy
      * [ ] LZ4
    * **Requests**
      * [x] Startup
      * [ ] Auth-Response
      * [x] Options
      * [ ] Query
      * [ ] Prepare
      * [ ] Execute
      * [ ] Batch
      * [ ] Register
    * **Responses**
      * [ ] Error
      * [x] Ready
      * [ ] Authenticate
      * [x] Supported
      * [ ] Event
      * [ ] Auth-Challenge
      * [ ] Auth-Success
      * **Result**
        * [ ] Void
        * [ ] Rows
        * [ ] Set-Keyspace
        * [ ] Prepared
        * [ ] Schema-Change
  * **Data Serialization Formats**
    * [ ] ascii
    * [ ] big-int
    * [ ] blob
    * [ ] boolean
    * [ ] decimal
    * [ ] double
    * [ ] float
    * [ ] inet
    * [ ] int
    * [ ] list
    * [ ] map
    * [ ] set
    * [ ] text
    * [ ] timestamp
    * [ ] uuid
    * [ ] varchar
    * [ ] varint
    * [ ] timeuuid
    * [ ] tuple
    * [ ] UDT (User Defined Type)
* **Codec V4**
  * **Messages**
    * [ ] Changes to the paging state of Result messages
    * [ ] Read_failure error code
    * **Resquests**
      * [ ] custom QueryHandler suppoert for QUERY, PREPARE, EXECUTE and BATCH
    * **Responses**
      * [ ] Warnings support
      * **Result**
        * [ ] New Schema-Change format
        * [ ] Prepared message includes partition key
  * **Data Serialization Formats**
    * [ ] date
    * [ ] time
    * [ ] tinyint
    * [ ] smallint
