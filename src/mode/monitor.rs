// CCheck: utility for scanning and probing minecraft servers
// Copyright (C) 2022 cleonyc

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Generexit_if_success: exit_on_successse as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
use anyhow::bail;
use owo_colors::OwoColorize;
use spinoff::{Color, Spinner, Spinners};
use std::panic;
use std::time::Duration as StdDuration;
use std::{net::IpAddr, sync::Arc};
use tokio::{net::TcpStream, sync::RwLock};
use webhook::{client::WebhookClient};

use crate::{adapters::CCheckResponse, condition::Conditions, format::ccheck::Server};

#[derive(Debug, Clone)]
pub struct Monitor {
    /// Servers per second to scan
    pub rate: usize,
    pub timeout: StdDuration,
    pub conditions: Conditions,
    pub addrs: Arc<RwLock<Vec<(IpAddr, u16)>>>,
    pub webhook_url: Option<String>,
}
impl Monitor {
    pub async fn ping(&self, server: usize) -> anyhow::Result<(CCheckResponse, Server)> {
        let addr = self.addrs.read().await[server].clone();
        // println!(
        //     "{} Pinging server {}",
        //     "::".blue().bold(),
        //     format!("{}:{}", addr.0, addr.1).cyan()
        // );
        let ip_cloned = addr.0.clone();
        panic::set_hook(Box::new(move |info| {
            eprintln!("panic!! info: {}, sv: {}:{}", info, ip_cloned, addr.1);
        }));
        let mut stream =
            match tokio::time::timeout(self.timeout, TcpStream::connect(addr.clone())).await {
                Ok(s) => match s {
                    Ok(s) => s,
                    Err(err) => {
                        bail!(err);
                    }
                },
                Err(err) => {
                    bail!(err);
                }
            };
        let resp = match tokio::time::timeout(
            self.timeout,
            craftping::tokio::ping(&mut stream, &addr.0.to_string(), addr.1),
        )
        .await
        {
            Ok(res) => match res {
                Ok(res) => res,
                Err(err) => {
                    bail!(err);
                }
            },
            Err(err) => {
                bail!(err);
            }
        };
        let cresp: CCheckResponse = resp.into();
        if self.conditions.is_valid(cresp.clone()) {
            if let Some(webhook) = self.webhook_url.clone() {
                let client = WebhookClient::new(&webhook.clone());
                client
                    .send(|m| {
                        m.embed(|e| {
                            e.title("Server matching conditions found!")
                                .description(&format!("`{}:{}`", addr.0, addr.1))
                                .field("Version", &cresp.clone().version, true)
                                .field(
                                    "Players",
                                    &cresp
                                        .clone()
                                        .sample
                                        .unwrap_or(vec![])
                                        .iter()
                                        .fold(String::new(), |str, val| {
                                            format!("{}, {str}", val.name)
                                        }),
                                    false,
                                )
                                .field("Motd", &cresp.clone().description.text, false)
                        })
                    })
                    .await
                    .expect("Failed to send webhook");
            }
            println!(
                "{} Found matching server {}",
                "::".green().bold(),
                format!("{}:{}", addr.0, addr.1).cyan()
            );
            return Ok((cresp.clone(), Server::from_resp(cresp.clone(), addr)));
        }
        bail!("Server does not match condtions")
    }
    pub async fn run(&self, exit_on_success: bool) -> anyhow::Result<()> {
        println!("{} Monitoring {} servers", "::".blue().bold(), self.addrs.read().await.len().purple());
        let mut spinner = Spinner::new(Spinners::Dots, "Monitoring servers", Color::Blue);
        loop {
            spinner.update_text("Monitoring servers");
            let mut join_handles = vec![];
            
            for sv in 0..self.addrs.read().await.len() {
                let self_clone = self.clone();
                let jh = tokio::spawn(async move { self_clone.ping(sv).await });
                join_handles.push(jh);
                tokio::time::sleep(StdDuration::from_micros((1_000_000 / self.rate) as u64)).await;
            }
            spinner.update_text("Processing results");
            let mut exit = false;
            for jh in join_handles {
                if let Ok(Ok(_)) = jh.await {
                    if exit_on_success {
                        
                        exit = true
                    }
                }
            }
            if exit {break}
        }
        spinner.success("Done!");
        Ok(())
    }
}
