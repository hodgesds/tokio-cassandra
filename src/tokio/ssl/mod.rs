#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TlsOptions;

#[cfg(feature = "with-openssl")]
pub mod ssl_client;
