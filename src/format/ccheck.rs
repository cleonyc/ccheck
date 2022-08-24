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
use std::{net::IpAddr, path::PathBuf, io::{BufWriter, Write, BufReader, Read}, fs::File};

use serde::{Serialize, Deserialize};
use time::OffsetDateTime;

use crate::adapters::{CCheckChat, CCheckResponse, CCheckPlayer};

pub struct CCheckFormat {
    pub servers: Vec<Server>,
}
impl CCheckFormat {
    pub fn new(servers: Vec<Server>) -> Self {
        CCheckFormat { servers }
    }
    pub fn save(&self, path: PathBuf) -> anyhow::Result<()> {
        let mut file = std::fs::File::create(path)?;
        let json = serde_json::to_string_pretty(&self.servers)?;
        let mut buf_write = BufWriter::new(&mut file);
        buf_write.write_all(json.as_bytes())?;
        Ok(())
    }
}
impl TryFrom<File> for CCheckFormat {
    type Error = anyhow::Error;
    fn try_from(file: File) -> anyhow::Result<Self> {
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content)?;
        let servers: Vec<Server> = serde_json::from_str(&content)?;
        Ok(CCheckFormat { servers })
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Server {
    version: String,
    protocol: usize,
    pub ip: (IpAddr, u16),
    players: Vec<Player>,
    favicon: String,
    motd: CCheckChat
}
impl Server {
    pub fn from_resp(resp: CCheckResponse, ip: (IpAddr, u16)) -> Self {
        let players = resp.sample.unwrap_or(vec!()).iter().map(|p| p.clone().into()).collect();
        let motd = resp.description;
        Server {
            version: resp.version,
            protocol: resp.protocol as usize,
            ip,
            players,
            favicon: base64::encode(resp.favicon.unwrap_or(b"".to_vec())),
            motd,
        }
    }
}
#[derive(Serialize, Deserialize, Clone)]
pub struct Player {
    last_online: usize,
    username: String,
    uuid: String,
}
impl From<CCheckPlayer> for Player {
    fn from(p: CCheckPlayer) -> Self {
        Player {
            last_online: OffsetDateTime::now_utc().unix_timestamp() as usize,
            username: p.name,
            uuid: p.id,
        }
    }
}
