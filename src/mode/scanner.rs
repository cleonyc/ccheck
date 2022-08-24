// CCheck: utility for scanning and probing minecraft servers
// Copyright (C) 2022 cleonyc

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
use anyhow::bail;
use craftping::{tokio::ping, Response};
use kdam::{prelude::BarMethods, tqdm, Bar};
use owo_colors::OwoColorize;
use std::{fmt::Display, net::IpAddr, sync::Arc, time::Duration, panic};
use time::OffsetDateTime;
use tokio::{
    net::TcpStream,
    sync::{mpsc::Sender, RwLock},
};

use crate::{adapters::CCheckResponse, condition::Conditions, format::ccheck::Server};
#[derive(Debug, Clone)]

pub struct Scanner {
    pub addrs: Arc<RwLock<Vec<(IpAddr, u16)>>>,
    pub conditions: Conditions,
    pub timeout: Duration,
}
#[derive(Debug)]
enum ScannerError {
    EmptyAddrs,
}
impl Display for ScannerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScannerError::EmptyAddrs => write!(f, "Empty Address buffer!"),
        }
    }
}
struct ServerInfo {
    version: String,
    address: IpAddr,
}
impl Scanner {
    async fn ping(&self, pb: Arc<RwLock<Bar>>) -> anyhow::Result<(CCheckResponse, Server)> {
        let addr = {
            if let Some(i) = self.addrs.write().await.pop() {
                i
            } else {
                bail!(ScannerError::EmptyAddrs)
            }
        };
        let mut stream =
            match tokio::time::timeout(self.timeout, TcpStream::connect(addr.clone())).await {
                Ok(s) => match s {
                    Ok(s) => s,
                    Err(err) => {
                        {
                            let mut bar = pb.write().await;
                            bar.update(1);
                            bar.write(format!(
                                "{} Bad server {}",
                                "::".red().bold(),
                                format!("{}:{}", addr.0, addr.1).cyan()
                            ));
                        }
                        bail!(err);
                    }
                },
                Err(err) => {
                    {
                        let mut bar = pb.write().await;
                        bar.update(1);
                        bar.write(format!(
                            "{} Bad server {}",
                            "::".red().bold(),
                            format!("{}:{}", addr.0, addr.1).cyan()
                        ));
                    }
                    bail!(err);
                }
            };
        
    
        return match 
            tokio::time::timeout(
                self.timeout,
                ping(&mut stream, &addr.0.to_string(), addr.1),
            )
            .await
        {
            Ok(res) => match res {
                Ok(res) => {
                    {
                        let mut bar = pb.write().await;
                        bar.update(1);
                        bar.write(format!(
                            "{} Good server {}",
                            "::".green().bold(),
                            format!("{}:{}", addr.0, addr.1).cyan()
                        ));
                    }
                    let c_res: CCheckResponse = res.into();
                    Ok((c_res.clone(), Server::from_resp(c_res, addr)))
                }
                Err(err) => {
                    {
                        let mut bar = pb.write().await;
                        bar.update(1);
                        bar.write(format!(
                            "{} Bad server {}",
                            "::".red().bold(),
                            format!("{}:{}", addr.0, addr.1).cyan()
                        ));
                    }
                    bail!(err)
                }
            },
            Err(err) => {
                {
                    let mut bar = pb.write().await;
                    bar.update(1);
                    bar.write(format!(
                        "{} Bad server {}",
                        "::".red().bold(),
                        format!("{}:{}", addr.0, addr.1).cyan()
                    ));
                }
                // pb.write().await.update(1);
                // println!("{} Bad server {}", "::".red().bold(), format!("{}:{}",  addr.0, addr.1).cyan());
                bail!(err)
            }
        };
    }
    pub async fn run(&self, interval: usize, start_time: usize) -> anyhow::Result<Vec<Server>> {
        let mut join_handles = vec![];
        let mut first = true;
        let pb = Arc::new(RwLock::new(tqdm!(
            total = self.addrs.read().await.len().clone(),
            // bar_format = "{animation} {percentage}".parse::<Template>().unwrap(),
            colour = "gradient(#5A56E0,#EE6FF8)",
            force_refresh = true
        )));
        loop {
            if self.addrs.read().await.len() == 0 {
                break;
            }
            let self_clone = self.clone();
            if first {
                if OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000
                    != start_time as i128
                {
                    continue;
                }
            }
            first = false;
            let new_pb = pb.clone();
            let jh = tokio::spawn(async move { self_clone.ping(new_pb).await });
            join_handles.push(jh);
            tokio::time::sleep(Duration::from_micros(interval as u64)).await;
        }
        let mut results = vec![];
        for jh in join_handles {
            match jh.await {
                Ok(res) => {
                    match res {
                        Ok(resp) => {
                            if self.conditions.clone().is_valid(resp.clone().0) {
                                results.push(resp.1)
                            }
                        },
                        Err(_) => {},
                    }
                }
                Err(_) => {}
            }
        }
        println!("{} Finished!", "::".green().bold());
        Ok(results)
    }
}
