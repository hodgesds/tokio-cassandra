#[cfg(feature = "with-openssl")]
use openssl::ssl::SslConnector;

#[derive(Clone)]
pub struct Options {
    /// The name of the domain whose certificate should be verified.
    /// It must match the domain name of the node to connect to. For example,
    /// `node01.cas.domain.com` would have `domain.com` as domain name.
    pub domain: String,
    pub configuration: Configuration,
}

/// All configuration required to quickly and easily setup an SSL/TLS connection
/// If all fields are None, a standard TLS configuration will be established against a trusted host.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct EasyConfiguration {
    /// The Path to the CA file containing trusted certificates in PEM format.
    /// It can be used to trust self-signed or otherwise untrusted certificates.
    pub certificate_authority_file: Option<String>,
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

#[derive(Clone)]
pub enum Configuration {
    /// Use a custom SslConnector to be used to setup a TLS connection.
    #[cfg(feature = "with-openssl")]
    Custom(SslConnector),

    /// Provide a set of credentials to use and automatically configure an SslConnector with.
    /// This can be considered easy-mode for the most common TLS cases.
    Predefined(EasyConfiguration),
}

#[cfg(feature = "with-openssl")]
#[doc(hidden)]
pub mod openssl_client;

#[cfg(feature = "with-openssl")]
pub use self::openssl_client as ssl_client;
