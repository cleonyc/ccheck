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
use std::{net::IpAddr, sync::Arc};

use std::time::Duration as StdDuration;
use tokio::sync::RwLock;

use crate::format::ccheck::CCheckFileHandler;
use crate::{
    condition::Conditions,
    mode::{monitor::Monitor, scanner::Scanner, Mode},
};
pub struct Config {
    pub mode: Mode,
    pub addrs: Vec<(IpAddr, u16)>,
    pub conditions: Conditions,
    pub timeout: StdDuration,
}
impl Config {
    pub async fn run(&self) -> anyhow::Result<()> {
        match &self.mode {
            Mode::Scanner {
                workers,
                output,
                progress_bar,
            } => {
                let scanner = Scanner {
                    timeout: self.timeout,
                    addrs: Arc::new(RwLock::new(self.addrs.clone())),
                    conditions: self.conditions.clone(),
                    progress_bar: *progress_bar,
                };
                let file_handler = CCheckFileHandler::new(output.to_path_buf()).await?;
                scanner.run(*workers, file_handler).await?;
            }
            Mode::Monitor {
                workers: rate,
                webhook_url,
                exit_on_success,
            } => {
                let monitor = Monitor {
                    rate: *rate,
                    timeout: self.timeout,
                    conditions: self.conditions.clone(),
                    addrs: Arc::new(RwLock::new(self.addrs.clone())),
                    webhook_url: webhook_url.clone(),
                };
                monitor.run(*exit_on_success).await?;
            }
        }
        Ok(())
    }
}
