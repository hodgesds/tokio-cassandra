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

pub fn run() -> Result<()> {
    env_logger::init().unwrap();

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
            .takes_value(true))
        .arg(Arg::with_name("password")
            .required(false)
            .short("p")
            .long("password")
            .takes_value(true)));
    let args: clap::ArgMatches = app.get_matches();
    match args.subcommand() {
        ("test-connection", Some(args)) => {
            let x = tcc::test_connection(args);
            println!("x = {:?}", x); 
            x
        },
        _ => {
            println!("{}", args.usage());
            ::std::process::exit(2);
        }
    }
}
