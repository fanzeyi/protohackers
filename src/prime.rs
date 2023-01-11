use anyhow::{anyhow, Context, Result};
use async_io::Async;
use asynchronous_codec::{Framed, LinesCodec};
use futures_lite::{AsyncReadExt, AsyncWriteExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::net::{SocketAddr, TcpStream};

use crate::utils::run_tcp_server_with;

fn split(stream: Async<TcpStream>) -> Result<(Async<TcpStream>, Async<TcpStream>)> {
    let inner = stream.into_inner().context("to get inner stream")?;

    Ok((Async::new(inner.try_clone()?)?, Async::new(inner)?))
}

#[derive(Deserialize, Debug)]
struct Request {
    method: String,
    number: f64,
}

#[derive(Serialize, Debug)]
struct Response {
    method: String,
    prime: bool,
}

impl Response {
    fn new(prime: bool) -> Self {
        Response {
            method: "isPrime".into(),
            prime,
        }
    }
}

fn is_prime(num: f64) -> bool {
    if num.fract() != 0.0 {
        return false;
    }
    let num = num.trunc() as i64;
    if num <= 1 {
        return false;
    }

    for i in 2..=((num as f64).sqrt().floor() as i64) {
        if num % i == 0 {
            return false;
        }
    }
    return true;
}

async fn handle_prime_request(outgoing: &mut Async<TcpStream>, line: String) -> Result<()> {
    let req = serde_json::from_str::<Request>(&line)?;
    if req.method != "isPrime" {
        return Err(anyhow!("invalid method name: '{}'", req.method));
    }
    log::info!("processing: '{:?}'", req);
    let result = Response::new(is_prime(req.number));
    let mut result = serde_json::to_string(&result)
        .with_context(|| format!("unable to serialize response: '{:?}'", result))?;
    result.push('\n');

    outgoing.write_all(result.as_bytes()).await?;

    Ok(())
}

pub async fn run(address: SocketAddr) -> Result<()> {
    log::info!("running prime");

    run_tcp_server_with(address, |stream, _addr| async move {
        let (read, mut write) = split(stream).expect("to split");
        let mut framed = Framed::new(read, LinesCodec {});
        while let Some(Ok(line)) = framed.next().await {
            if let Err(e) = handle_prime_request(&mut write, line).await {
                log::info!("error while handling request: '{:?}'", e);
                write.write_all(b"{}\n").await.ok();
                break;
            }
        }
    })
    .await
    .context("unable to create tcp server")?;

    Ok(())
}
