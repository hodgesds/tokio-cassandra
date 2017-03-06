use std::io;
use std::sync::Arc;
use std::net::SocketAddr;
use std::marker::PhantomData;

use tokio_proto::BindClient;
use tokio_core::reactor::Handle;
use tokio_core::net::TcpStream;
use tokio_openssl::{SslStream, SslConnectorExt};
use tokio::utils::io_err;
use futures::{Future, Poll, Async, future};
use openssl::pkcs12::Pkcs12;
use openssl::x509::X509;
use openssl::ssl::{SslMethod, SslConnectorBuilder};

use super::{Credentials, Options};

pub struct SslClient<Kind, P> {
    _kind: PhantomData<Kind>,
    proto: Arc<P>,
    tls: Options,
}

pub struct Connect<Kind, P> {
    _kind: PhantomData<Kind>,
    proto: Arc<P>,
    socket: Box<Future<Item = SslStream<TcpStream>, Error = io::Error>>,
    handle: Handle,
}

impl<Kind, P> Future for Connect<Kind, P>
    where P: BindClient<Kind, SslStream<TcpStream>>
{
    type Item = P::BindClient;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<P::BindClient, io::Error> {
        let socket = try_ready!(self.socket.poll().map_err(|e| io::Error::new(io::ErrorKind::Other, e)));
        Ok(Async::Ready(self.proto.bind_client(&self.handle, socket)))
    }
}

impl<Kind, P> SslClient<Kind, P>
    where P: BindClient<Kind, SslStream<TcpStream>>
{
    pub fn new(protocol: P, tls: Options) -> SslClient<Kind, P> {
        SslClient {
            _kind: PhantomData,
            proto: Arc::new(protocol),
            tls: tls,
        }
    }

    pub fn connect(&self, addr: &SocketAddr, handle: &Handle) -> Connect<Kind, P> {
        Connect {
            _kind: PhantomData,
            proto: self.proto.clone(),
            socket: {
                let tls = self.tls.clone();
                Box::new(TcpStream::connect(addr, handle).and_then(|stream| {
                    future::done(SslConnectorBuilder::new(SslMethod::tls()))
                        .map_err(io_err)
                        .and_then(move |mut connector| {
                            let domain = tls.domain;
                            let ca_file = tls.certificate_authority_file;
                            let res = match tls.credentials {
                                    Some(Credentials::Pk12 { contents, passphrase }) => {
                                        Pkcs12::from_der(&contents)
                                            .and_then(|p| p.parse(&passphrase))
                                            .and_then(|identity| {
                                                let builder = connector.builder_mut();
                                                builder.set_private_key(&identity.pkey)
                                                    .and_then(|_| builder.set_certificate(&identity.cert))
                                                    .and_then(|_| builder.check_private_key())
                                                    .and_then(move |_| {
                                                        // TODO: remove this once this is available upstream:
                                                        // https://github.com/sfackler/rust-openssl/pull/592
                                                        if identity.chain.len() as isize == -1 {
                                                            return Ok(());
                                                        }
                                                        for cert in identity.chain {
                                                            builder.add_extra_chain_cert(cert)?
                                                        }
                                                        Ok(())
                                                    })
                                            })
                                            .map_err(io_err)
                                            .map(|_| connector)
                                    }
                                    _ => Ok(connector),
                                }
                                .and_then(move |mut connector| match ca_file {
                                    Some(fp) => {
                                        connector.builder_mut()
                                            .set_ca_file(&fp)
                                            .map_err(|e| {
                                                format!("Failed to use certificate-authority \
                                                         file at '{}' with error: {}",
                                                        fp,
                                                        e)
                                            })
                                            .map_err(io_err)
                                            .and_then(|_| {
                                                use ::std::fs::File;
                                                use ::std::io::Read;
                                                let mut f = File::open(&fp).unwrap();
                                                let mut buf = Vec::new();
                                                f.read_to_end(&mut buf).unwrap();
                                                X509::from_pem(&buf)
                                                    .map_err(|e| {
                                                        format!("Failed to read PEM file with trusted \
                                                         certificates at '{}' with errror: {}",
                                                                fp,
                                                                e)
                                                    })
                                                    .map_err(io_err)
                                                .and_then(|x509|
                                                    connector.builder_mut().cert_store_mut().add_cert(x509)
                                                    .map_err(io_err))
                                            })
                                            .map_err(io_err)
                                            .map(|_| connector)
                                    }
                                    None => Ok(connector),
                                });
                            future::done(res)
                                .map(|c| c.build())
                                .and_then(move |connector| {
                                    connector.connect_async(&domain, stream)
                                        .map_err(io_err)
                                })
                        })
                }))
            },
            handle: handle.clone(),
        }
    }
}
