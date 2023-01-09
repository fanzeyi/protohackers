use std::ops::Bound::Included;
use std::{collections::BTreeMap, net::SocketAddr};

use anyhow::Result;
use futures_lite::{AsyncReadExt, AsyncWriteExt};
use smol::spawn;

use crate::utils::run_tcp_server_with;

struct State {
    map: BTreeMap<i32, i32>,
}

impl State {
    fn new() -> Self {
        State {
            map: BTreeMap::new(),
        }
    }

    fn insert(&mut self, timestamp: i32, price: i32) {
        self.map.insert(timestamp, price);
    }

    fn query(&self, min: i32, max: i32) -> Result<i32> {
        if min > max {
            return Ok(0);
        }

        let prices = self.map.range((Included(&min), Included(&max)));
        let count = prices.clone().count() as i128;
        let sum: i128 = prices.map(|(_, &v)| v as i128).sum();

        if count == 0 {
            return Ok(0);
        }

        let mean = sum / count;

        Ok(mean.try_into()?)
    }
}

pub async fn run(address: SocketAddr) -> Result<()> {
    run_tcp_server_with(address, |mut stream, _addr| {
        spawn(async move {
            let mut request = [0; 9];
            let mut state = State::new();

            while let Ok(_) = stream.read_exact(&mut request).await {
                log::debug!("raw bytes: {request:02X?}");
                let ops = char::from(request[0]);
                let lhs = i32::from_be_bytes(request[1..5].try_into().expect("correct size"));
                let rhs = i32::from_be_bytes(request[5..9].try_into().expect("correct size"));
                log::info!("received request op='{ops}' lhs='{lhs}' rhs='{rhs}'");

                match ops {
                    'I' => state.insert(lhs, rhs),
                    'Q' => {
                        let result = state.query(lhs, rhs).unwrap_or(0).to_be_bytes();
                        stream.write_all(&result).await.ok();
                    }
                    _ => break,
                }
            }
        })
        .detach();
    })
    .await?;
    Ok(())
}
