#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TlsOptions;

#[cfg(feature = "ssl")]
pub mod ssl_client;
