extern crate tcc;

extern crate futures;
extern crate tokio_core;
extern crate tokio_service;

#[macro_use]
extern crate clap;

#[macro_use]
extern crate error_chain;

use clap::{SubCommand, Arg};
use futures::Future;
use tokio_core::reactor::Core;
use tokio_service::Service;
//use tokio_cassandra::tokio::Client;

use tcc::errors::*;

quick_main!(run);

pub fn run() -> Result<()> {

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
            .help("The port to connect to")));
    let args: clap::ArgMatches = app.get_matches();
    match args.subcommand() {
        ("test-connection", Some(args)) => tcc::test_connection(args),
        _ => {
            println!("{}", args.usage());
            ::std::process::exit(2);
        }
    }

    //    let mut core = Core::new().unwrap();
    //    let handle = core.handle();
    //    let addr = "127.0.0.1".parse().unwrap();
    //    core.run(line::Client::connect(&addr, &handle).and_then(|client| {
    //            client.call("Hello".to_string())
    //                .and_then(move |response| {
    //                    println!("CLIENT: {:?}", response);
    //                    client.call("Goodbye".to_string())
    //                })
    //                .and_then(|response| {
    //                    println!("CLIENT: {:?}", response);
    //                    Ok(())
    //                })
    //        }))
    //        .unwrap();
}
