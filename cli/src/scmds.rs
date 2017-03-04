use clap;
use super::args::ConnectionOptions;
use super::errors::*;

pub fn test_connection(opts: ConnectionOptions, _args: &clap::ArgMatches) -> Result<()> {
    let addr = opts.addr.clone();
    let (mut core, client) = opts.connect();
    core.run(client)
        .chain_err(|| format!("Failed to connect to {}", addr))
        .map(|_response| {
            println!("Connection to {} successful", addr);
            ()
        })
        .map_err(|e| e.into())
}
