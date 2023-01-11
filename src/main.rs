use std::{net::SocketAddr, str::FromStr};

use anyhow::{Context, Result};
use argh::FromArgs;
use futures_lite::FutureExt;

mod chat;
mod means;
mod prime;
mod smoke;
mod utils;

#[derive(FromArgs)]
/// Protohackers binary
struct Options {
    #[argh(positional)]
    kind: String,

    #[argh(switch)]
    /// if the server should accept request from other hosts
    public: bool,
}

fn main() -> Result<()> {
    env_logger::init();
    let options: Options = argh::from_env();

    let address = SocketAddr::from_str(if options.public {
        "0.0.0.0:45962"
    } else {
        "127.0.0.1:4000"
    })
    .context("unable to parse network address")?;

    let fut = match options.kind.as_ref() {
        "smoke" => crate::smoke::run(address).boxed(),
        "prime" => crate::prime::run(address).boxed(),
        "means" => crate::means::run(address).boxed(),
        "chat" => crate::chat::run(address).boxed(),
        _ => panic!("invalid choice"),
    };

    smol::block_on(fut)?;

    Ok(())
}
