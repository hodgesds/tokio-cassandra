extern crate tcc;

extern crate futures;
extern crate tokio_core;
extern crate tokio_service;

#[macro_use]
extern crate clap;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate env_logger;

use clap::{SubCommand, Arg};

use tcc::errors::*;

quick_main!(run);

arg_enum!{
    #[derive(Debug)]
    pub enum CertKind {
        PK12
    }
}

pub fn run() -> Result<()> {
    env_logger::init().unwrap();
    let default_cert_type = format!("{}", CertKind::PK12);

    let mut app: clap::App = app_from_crate!();
    app = app.subcommand(SubCommand::with_name("test-connection")
        .arg(Arg::with_name("host")
            .required(true)
            .takes_value(true)
            .help("The name or IP address of the host to connect to."))
        .arg(Arg::with_name("port")
            .required(false)
            .default_value("9042")
            .takes_value(true)
            .help("The port to connect to"))
        .arg(Arg::with_name("user")
            .required(false)
            .short("u")
            .long("user")
            .takes_value(true)
            .help("The name of the user to login authenticate as"))
        .arg(Arg::with_name("password")
            .required(false)
            .short("p")
            .long("password")
            .takes_value(true)
            .help("The user's password. Please note that the password might persist in your \
                   history file if provided here"))
        .arg(Arg::with_name("tls")
            .required(false)
            .takes_value(false)
            .long("tls")
            .help("Encrypt the connection via TLS. This will never connect via plain-text, \
                   even if the server supports that too."))
        .arg(Arg::with_name("cert-type")
            .required(false)
            .takes_value(true)
            .long("cert-type")
            .default_value(&default_cert_type)
            .help("Encrypt the connection via TLS. This will never connect via plain-text, \
                   even if the server supports that too."))
        .arg(Arg::with_name("cert")
            .required(false)
            .takes_value(true)
            .long("cert")
            .help("The path to the certificate file in a format defined by --cert-type. A \
                   password can be provided by separating it with a colon, such as in \
                   /path/to/cert:password.")));
    let args: clap::ArgMatches = app.get_matches();
    match args.subcommand() {
        ("test-connection", Some(args)) => tcc::test_connection(args),
        _ => {
            println!("{}", args.usage());
            ::std::process::exit(2);
        }
    }
}
