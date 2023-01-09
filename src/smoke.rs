use std::net::SocketAddr;

use anyhow::{Context, Result};
use futures_lite::{AsyncReadExt, AsyncWriteExt};

use crate::utils::run_tcp_server_with;

pub async fn run(address: SocketAddr) -> Result<()> {
    log::info!("running smoke");

    run_tcp_server_with(address, |mut stream, _addr| {
        smol::spawn(async move {
            let mut buffer = Vec::new();
            stream.read_to_end(&mut buffer).await.ok();
            stream.write_all(&buffer).await.ok();
        })
        .detach();
    })
    .await
    .context("unable to create tcp server")?;

    Ok(())
}
