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
use std::io::{BufReader, Read};

use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use crate::condition::{Actor, Condition, ConditionType, Conditions};
use crate::format::ccheck::CCheckFormat;
use crate::format::masscan::{MasscanFormat};
// use crate::input::{Port, Server};
use crate::mode::Mode;

use super::Config;
use anyhow::bail;
use regex::Regex;
use toml_edit::{Array, Document};

pub struct TomlFormat {
    pub file: File,
}
impl TryFrom<TomlFormat> for Config {
    type Error = anyhow::Error;

    fn try_from(toml: TomlFormat) -> Result<Self, anyhow::Error> {
        let mut buf_reader = BufReader::new(toml.file);
        let mut content = String::new();
        buf_reader.read_to_string(&mut content)?;
        let doc = content.parse::<Document>().expect("Invalid config file!");
        let mode = match doc["mode"].as_str().expect("invalid mode") {
            "monitor" => crate::mode::Mode::Monitor {
                rate: doc["monitor"]["rate"].as_integer().expect("invalid rate") as usize,
                webhook_url: match doc["monitor"]["webhook_url"].as_str() {
                    Some(str) => {Some(str.to_string())},
                    None => {None},
                },
                exit_on_success: doc["monitor"]["exit_on_success"].as_bool().unwrap_or(true),
            },
            "scanner" => crate::mode::Mode::Scanner {
                rate: doc["scanner"]["rate"]
                    .as_integer()
                    .expect("no rate specified") as usize,
                output: PathBuf::from_str(doc["scan"]["output"].as_str().expect("output not specified"))?
            },
            _ => {
                bail!("invalid mode")
            }
        };
        Ok(Self {
            timeout: Duration::from_millis(doc["timeout"].as_integer().unwrap_or(500) as u64),
            mode: mode.clone(),
            addrs: match mode.clone() {
                Mode::Scanner { rate: _, output: _ } => {
                    let masscan_file = File::open(
                        doc["scan"]["input"]
                            .as_str()
                            .expect("invalid input file specified"),
                    )
                    .expect("invalid file specified");
                    let masscan_input = MasscanFormat::try_from(masscan_file)?;
                    masscan_input.get_ips()
                }
                Mode::Monitor {
                    rate: _,
                    webhook_url: _,
                    exit_on_success: _,
                } => {
                    if let Some(mscan) = doc["monitor"]["masscan_input"].as_str() {
                        let masscan_file = File::open(mscan).expect("invalid file specified");
                        let masscan_input = MasscanFormat::try_from(masscan_file)?;
                        masscan_input.get_ips()
                    } else {
                        let ccheck_file = File::open(
                            doc["monitor"]["ccheck_input"]
                                .as_str()
                                .expect("invalid input file specified"),
                        )
                        .expect("invalid file specified");
                        let ccheck_input = CCheckFormat::try_from(ccheck_file)?;
                        ccheck_input.servers.iter().map(|sv| sv.ip ).collect()
                    }
                }
            },
            conditions: {
                Conditions {
                    conditions: {
                        let mut out = vec![];
                        for table in doc["conditions"]
                            .clone()
                            .into_table()
                            .expect("invalid conditions")
                            .iter()
                        {
                            let actor = Actor::from_str(table.0)
                                .expect(&format!("invalid condition parameter: `{}`", table.0));
                            let table = table.1.clone();
                            let default = &Array::new();
                            let excludes: Vec<String> = table["excludes"]
                                .as_array()
                                .unwrap_or(default)
                                .iter()
                                .map(|val| {
                                    val.as_str()
                                        .expect(&format!("invalid excludes: {}", val))
                                        .to_string()
                                })
                                .collect();
                            let includes: Vec<String> = table["includes"]
                                .as_array()
                                .unwrap_or(default)
                                .iter()
                                .map(|val| {
                                    val.as_str()
                                        .expect(&format!("invalid includes: {}", val))
                                        .to_string()
                                })
                                .collect();
                            let includes_regex: Vec<Regex> = table["regex.includes"]
                                .as_array()
                                .unwrap_or(default)
                                .iter()
                                .map(|val| {
                                    Regex::new(
                                        val.as_str()
                                            .expect(&format!("invalid includes regex: {}", val)),
                                    )
                                    .expect(&format!("invalid includes regex: {val}"))
                                })
                                .collect();
                            let excludes_regex: Vec<Regex> = table["regex.excludes"]
                                .as_array()
                                .unwrap_or(default)
                                .iter()
                                .map(|val| {
                                    Regex::new(
                                        val.as_str()
                                            .expect(&format!("invalid excludes regex: {}", val)),
                                    )
                                    .expect(&format!("invalid excludes regex: {val}"))
                                })
                                .collect();
                            out.append(&mut get_conditions(
                                actor,
                                excludes,
                                includes,
                                excludes_regex,
                                includes_regex,
                            ));
                        }

                        out
                    },
                }
            },
        })
    }
}

fn get_conditions(actor: Actor, excludes: Vec<String>, includes: Vec<String>, excludes_regex: Vec<Regex>, includes_regex: Vec<Regex>) -> Vec<Condition> {
    let mut out = Vec::new();
    if excludes_regex.len() > 0 || excludes.len() > 0 {
        out.push(Condition {
            values: excludes,
            values_regex: excludes_regex,
            conditon_type: ConditionType::Exclude,
            actor: actor.clone(),
        });
    }
    if includes_regex.len() > 0 || includes.len() > 0 {
        out.push(Condition {
            values: includes,
            values_regex: includes_regex,
            conditon_type: ConditionType::Include,
            actor: actor.clone(),
        });
    }
    return out;

}