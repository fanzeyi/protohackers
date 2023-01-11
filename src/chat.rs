use std::{
    collections::HashMap,
    net::{SocketAddr, TcpStream},
    sync::Arc,
};

use anyhow::{anyhow, Context, Result};
use async_io::Async;
use asynchronous_codec::{FramedRead, LinesCodec};
use futures_lite::{io::BufReader, AsyncBufReadExt, AsyncWriteExt, StreamExt};
use smol::lock::RwLock;

use crate::utils::{dup, run_tcp_server_with};

async fn welcome(stream: &mut Async<TcpStream>) -> Result<String> {
    stream
        .write_all(b"Welcome to budgetchat! What shall I call you? \n")
        .await?;

    let mut read = BufReader::new(stream);

    let mut username = String::new();
    read.read_line(&mut username).await.ok();

    let username = username.trim();
    let stream = read.into_inner();

    if username.len() < 1 || username.len() > 64 {
        stream
            .write_all(b"username too short or too long, bye.\n")
            .await?;
        return Err(anyhow!(""));
    }

    if username.chars().any(|c| !c.is_ascii_alphanumeric()) {
        stream
            .write_all(b"username contains invalid sequence, bye.\n")
            .await?;
        return Err(anyhow!(""));
    }

    Ok(username.to_string())
}
struct Chatroom {
    users: RwLock<HashMap<String, Async<TcpStream>>>,
}

impl Chatroom {
    fn new() -> Self {
        Self {
            users: RwLock::new(HashMap::new()),
        }
    }

    async fn add_user(&self, username: String, stream: Async<TcpStream>) {
        self.users.write().await.insert(username, stream);
    }

    async fn remove_user(&self, username: &str) {
        let mut users = self.users.write().await;
        users.remove(username);
    }

    async fn exists(&self, username: &str) -> bool {
        self.users.read().await.contains_key(username)
    }

    async fn existing(&self) -> Vec<String> {
        self.users.read().await.keys().map(|s| s.clone()).collect()
    }

    async fn broadcast(&self, message: String, exclude: Option<&str>) {
        let users = self.users.read().await;

        for (u, mut st) in users.iter() {
            if let Some(exclude) = exclude.as_deref() {
                if exclude == u {
                    continue;
                }
            }
            st.write_all(message.as_bytes()).await.ok();
        }
    }

    async fn announce(&self, username: &str) {
        log::info!("announcing {}", username);
        self.broadcast(format!("* {} has entered the room\n", username), None)
            .await;
    }
}

pub async fn run(address: SocketAddr) -> Result<()> {
    let room = Arc::new(Chatroom::new());
    log::info!("running smoke");

    run_tcp_server_with(address, {
        move |stream, _addr| {
            let room = room.clone();
            async move {
                let (mut read, write) = dup(stream).expect("split stream");
                let username = match welcome(&mut read).await {
                    Err(e) => {
                        log::info!("encounter error when processing stream: {:?}", e);
                        return;
                    }
                    Ok(username) => username,
                };

                if room.exists(&username).await {
                    read.write_all(b"username is already taken\n").await.ok();
                    return;
                }

                // welcome sequence
                room.announce(&username).await;
                let existing = room.existing().await;
                let message = format!("* The room contains: {}\n", existing.join(", "));
                room.add_user(username.clone(), write).await;
                read.write_all(message.as_bytes()).await.ok();

                let mut incoming = FramedRead::new(read, LinesCodec {});
                while let Some(Ok(line)) = incoming.next().await {
                    let message = format!("[{}] {}", username, line);
                    room.broadcast(message, Some(&username)).await;
                }

                room.remove_user(&username).await;
                room.broadcast(format!("* {} has left the room\n", username), None)
                    .await;
            }
        }
    })
    .await
    .context("unable to create tcp server")?;

    Ok(())
}
