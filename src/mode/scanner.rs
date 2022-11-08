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
use craftping::tokio::ping;
use kdam::{tqdm, Bar, BarExt};
use owo_colors::OwoColorize;
use spinoff::{Color, Spinner, Spinners};
use std::{fmt::Display, net::IpAddr, sync::Arc, time::Duration};
use tokio::{net::TcpStream, sync::RwLock};

use crate::{
    adapters::CCheckResponse,
    condition::Conditions,
    format::ccheck::{CCheckFileHandler, Server},
};
#[derive(Debug, Clone)]

pub struct Scanner {
    pub addrs: Arc<RwLock<Vec<(IpAddr, u16)>>>,
    pub conditions: Conditions,
    pub timeout: Duration,
    pub progress_bar: bool,
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
impl Scanner {
    async fn ping(&self, pb: Option<Arc<RwLock<Bar>>>) -> anyhow::Result<(CCheckResponse, Server)> {
        let addr = {
            if let Some(i) = self.addrs.write().await.pop() {
                i
            } else {
                bail!(ScannerError::EmptyAddrs)
            }
        };
        let mut stream = match tokio::time::timeout(self.timeout, TcpStream::connect(addr)).await {
            Ok(s) => match s {
                Ok(s) => s,
                Err(err) => {
                    update(pb).await;
                    bail!(err);
                }
            },
            Err(err) => {
                update(pb).await;
                bail!(err);
            }
        };

        match tokio::time::timeout(self.timeout, ping(&mut stream, &addr.0.to_string(), addr.1))
            .await
        {
            Ok(res) => match res {
                Ok(res) => {
                    update(pb).await;
                    let c_res: CCheckResponse = res.into();
                    Ok((c_res.clone(), Server::from_resp(c_res, addr)))
                }
                Err(err) => {
                    update(pb).await;
                    bail!(err)
                }
            },
            Err(err) => {
                update(pb).await;
                bail!(err)
            }
        }
    }
    pub async fn run(&self, workers: usize, out: CCheckFileHandler) -> anyhow::Result<()> {
        let mut join_handles = vec![];
        let total_servers = self.addrs.read().await.len();
        let pb = if self.progress_bar {
            Some(Arc::new(RwLock::new(tqdm!(
                total = self.addrs.read().await.len(),
                // bar_format = "{animation} {percentage}".parse::<Template>().unwrap(),
                colour = "gradient(#5A56E0,#EE6FF8)",
                force_refresh = true
            ))))
        } else {
            None
        };
        let spinner = if !self.progress_bar {
            Some(Spinner::new(Spinners::Dots, "Scanning", Color::Magenta))
        } else {
            None
        };
        let safe_file_handler = Arc::new(RwLock::new(out));
        for _ in 0..=workers {
            let self_clone = self.clone();
            let new_pb = pb.clone();
            let cloned_safe_file_handler = safe_file_handler.clone();
            let jh = tokio::spawn(async move {
                loop {
                    match self_clone.ping(new_pb.clone()).await {
                        Ok(resp) => {
                            cloned_safe_file_handler
                                .clone()
                                .write()
                                .await
                                .write_resp(resp.0)
                                .await
                                .unwrap();
                        }
                        Err(e) => {
                            if let Ok(ScannerError::EmptyAddrs) = e.downcast() {
                                break;
                            }
                        }
                    }
                }
            });
            join_handles.push(jh);
        }
        for jh in join_handles {
            jh.await?;
        }
        let good_servers = {
            let mut fh = safe_file_handler.write().await;
            fh.done().await?;
            fh.count
        };
        if let Some(s) = spinner {
            s.success(&format!(
                "Found {} good servers out of {}!",
                good_servers.cyan(),
                total_servers.cyan()
            ));
        } else {
            println!(
                "{} Found {} good servers out of {}!",
                "::".green().bold(),
                good_servers.cyan(),
                total_servers.cyan()
            );
        }

        Ok(())
    }
}
async fn update(opt_pb: Option<Arc<RwLock<Bar>>>) {
    if let Some(pb) = opt_pb {
        let mut bar = pb.write().await;
        bar.update(1);
    }
}
