#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Options {
    /// The name of the domain whose certificate should be verified.
    /// It must match the domain name of the node to connect to. For example,
    /// `node01.cas.domain.com` would have `domain.com` as domain name.
    pub domain: String,

    pub credentials: Option<Credentials>,
}

/// Various ways to specify credentials to allow clients to authenticate towards the server.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Credentials {
    Pk12 {
        contents: Vec<u8>,
        passphrase: String,
    },
}

#[cfg(feature = "with-openssl")]
#[doc(hidden)]
pub mod openssl_client;

#[cfg(feature = "with-openssl")]
pub use self::openssl_client as ssl_client;
