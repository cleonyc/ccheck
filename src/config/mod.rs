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
use std::{net::IpAddr, sync::Arc, path::PathBuf};

use time::{Duration, OffsetDateTime};
use tokio::sync::RwLock;
use std::time::Duration as StdDuration;

use crate::{
    condition::Conditions,
    mode::{scanner::Scanner, Mode, monitor::Monitor}, format::ccheck::CCheckFormat,
};
pub mod toml;
pub struct Config {
    pub mode: Mode,
    pub addrs: Vec<(IpAddr, u16)>,
    pub conditions: Conditions,
    pub timeout: StdDuration,
}
impl Config {
    pub async fn run(&self) -> anyhow::Result<()> {
        match &self.mode {
            Mode::Scanner { rate, output } => {
                let now = OffsetDateTime::now_utc();
                let start = (now
                    .replace_nanosecond(0)?
                    .replace_microsecond(0)?
                    .replace_millisecond(0)?
                    + Duration::seconds(1))
                .unix_timestamp_nanos()
                    / 1_000_000;
                let scanner = Scanner {
                    timeout: self.timeout,
                    addrs: Arc::new(RwLock::new(self.addrs.clone())),
                    conditions: self.conditions.clone(),
                };
                let svs = scanner
                    .run(1_000_000 / rate, start as usize)
                    .await?;
                let format = CCheckFormat::new(svs);
                format.save(output.clone())?;
            }
            Mode::Monitor { rate, webhook_url, exit_on_success } => {
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
