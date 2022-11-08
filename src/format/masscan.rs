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

use std::{
    fs::File,
    io::{BufReader, Read},
    net::IpAddr,
};

use serde::{Deserialize, Serialize};
pub struct MasscanFormat {
    pub servers: Vec<MasscanServer>,
}
impl MasscanFormat {
    pub fn get_ips(&self) -> Vec<(IpAddr, u16)> {
        self.servers
            .iter()
            .map(|s| {
                (
                    s.ip.clone().parse::<IpAddr>().expect("bad ip"),
                    s.ports[0].port,
                )
            })
            .collect()
    }
}
impl TryFrom<File> for MasscanFormat {
    type Error = anyhow::Error;

    fn try_from(file: File) -> anyhow::Result<Self> {
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content)?;
        let mut servers: Vec<MasscanServer> = serde_json::from_str(&content)?;
        servers.retain(|s| s.ports[0].service.is_none());
        Ok(MasscanFormat { servers })
    }
}
#[derive(Deserialize, Serialize)]
pub struct MasscanServer {
    pub ip: String,
    pub timestamp: String,
    pub ports: Vec<MasscanPort>,
}
#[derive(Deserialize, Serialize)]
pub struct MasscanPort {
    pub port: u16,
    pub proto: String,
    pub status: Option<String>,
    pub reason: Option<String>,
    pub service: Option<MasscanService>,
    pub ttl: Option<usize>,
}
#[derive(Deserialize, Serialize)]
pub struct MasscanService {
    pub name: String,
    pub banner: String,
}
