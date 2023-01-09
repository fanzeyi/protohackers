use std::net::{SocketAddr, TcpListener, TcpStream};

use anyhow::{Context, Result};
use async_io::Async;

pub async fn run_tcp_server_with<H>(address: SocketAddr, handle: H) -> Result<()>
where
    H: Fn(Async<TcpStream>, SocketAddr) -> (),
{
    let server = Async::<TcpListener>::bind(address)
        .with_context(|| format!("unable to bind to '{}'", address))?;

    log::info!("listening at '{}'", address);
    while let Ok((stream, peer)) = server.accept().await {
        handle(stream, peer);
    }

    Ok(())
}
