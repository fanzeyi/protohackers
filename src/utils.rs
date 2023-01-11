use std::future::Future;
use std::net::{SocketAddr, TcpListener, TcpStream};

use anyhow::{Context, Result};
use async_io::Async;

pub async fn run_tcp_server_with<R, H>(address: SocketAddr, handle: H) -> Result<()>
where
    R: Future<Output = ()> + Send + Sync + 'static,
    H: (Fn(Async<TcpStream>, SocketAddr) -> R) + Send + Sync + 'static,
{
    let server = Async::<TcpListener>::bind(address)
        .with_context(|| format!("unable to bind to '{}'", address))?;

    log::info!("listening at '{}'", address);
    while let Ok((stream, peer)) = server.accept().await {
        smol::spawn(handle(stream, peer)).detach();
    }

    Ok(())
}
