use std::net::{SocketAddr, TcpListener};

use anyhow::{Context, Result};
use async_io::Async;
use futures_lite::{AsyncReadExt, AsyncWriteExt};

pub async fn run(address: SocketAddr) -> Result<()> {
    log::info!("running smoke");
    let server = Async::<TcpListener>::bind(address)
        .with_context(|| format!("unable to bind to '{}'", address))?;

    log::info!("listening at '{}'", address);
    while let Ok((mut stream, peer)) = server.accept().await {
        log::info!("accepted client from '{}'", peer);
        smol::spawn(async move {
            let mut buffer = Vec::new();
            stream.read_to_end(&mut buffer).await.ok();
            stream.write_all(&buffer).await.ok();
        })
        .detach();
    }
    Ok(())
}
