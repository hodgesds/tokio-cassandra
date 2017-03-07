mod query {
    use clap;
    use super::super::args::ConnectionOptions;
    use super::super::errors::*;
    use tokio_cassandra::codec::primitives::{CqlFrom, CqlLongString};

    struct Options;

    impl Options {
        fn try_from(_args: &clap::ArgMatches) -> Result<Options> {
            Ok(Options)
        }

        fn try_into_query_string(self) -> Result<String> {
            Ok(String::new())
        }
    }

    pub fn query(opts: ConnectionOptions, args: &clap::ArgMatches) -> Result<()> {
        let addr = format!("{}:{}", opts.host, opts.port);
        let query = Options::try_from(args)?.try_into_query_string()?;
        let (mut core, client) = opts.connect();
        core.run(client)
            .chain_err(|| format!("Failed to connect to {}", addr))
            .and_then(|_client| {
                if args.is_present("dry-run") {
                    println!("{}", query);
                } else {
                    let _query = CqlLongString::<Vec<u8>>::try_from(&query)?;
                    unimplemented!();
                }
                Ok(())
            })
            .map_err(|e| e.into())
    }
}

mod testcon {
    use clap;
    use super::super::args::ConnectionOptions;
    use super::super::errors::*;

    pub fn test_connection(opts: ConnectionOptions, _args: &clap::ArgMatches) -> Result<()> {
        let addr = format!("{}:{}", opts.host, opts.port);
        let (mut core, client) = opts.connect();
        core.run(client)
            .chain_err(|| format!("Failed to connect to {}", addr))
            .map(|_response| {
                println!("Connection to {} successful", addr);
                ()
            })
            .map_err(|e| e.into())
    }
}

pub use self::testcon::*;
pub use self::query::*;
