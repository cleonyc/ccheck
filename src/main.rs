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
use std::{fs::File, path::PathBuf, str::FromStr, time::Duration};

use anyhow::bail;
use clap::{Parser, Subcommand};
use format::ccheck::CCheckFormat;
use regex::Regex;

use crate::{
    condition::{Actor, Condition, ConditionType, Conditions},
    config::Config,
    format::masscan::MasscanFormat,
    mode::Mode,
};

pub mod adapters;
pub mod condition;
pub mod config;
pub mod format;
pub mod mode;
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Option<Command>,
    /// Masscan config file to use to import settings and conditionals
    #[clap(value_parser, short, long)]
    config: Option<PathBuf>,
}
#[derive(Subcommand, Debug)]
enum Command {
    /// Use to scan for minecraft servers
    Scan {
        /// JSON file outputed by `masscan`
        #[clap(value_parser)]
        input: PathBuf,
        /// Number of async tasks to scan with.
        #[clap(short, long, value_parser, default_value_t = 30)]
        workers: usize,
        /// Formatted output file
        #[clap(value_parser)]
        output: PathBuf,
        /// Timeout for each server in milliseconds
        /// Default: 1000
        #[clap(short, long, value_parser, default_value_t = 1000)]
        timeout: u64,

        /// Use progress bar: slows down by a decent bit but has pretty output
        #[clap(short, long, value_parser, default_value_t = false)]
        progress_bar: bool,
        /// conditions to filter out servers
        /// format: `<actor>:<value>,<actor>:<value>`
        /// supported actors: `PlayerName, PlayerUuid, Version, Protocol, ConnectedPlayers, MaxPlayers, Description, Favicon (base64 encoded)`
        #[clap(long, value_parser)]
        exclude: Option<Vec<String>>,
        /// regex conditions to filter out servers
        /// format: `<actor>:<value>,<actor>:<value>`
        /// supported actors: `PlayerName, PlayerUuid, Version, Protocol, ConnectedPlayers, MaxPlayers, Description, Favicon (base64 encoded)`
        #[clap(long, value_parser)]
        exclude_regex: Option<Vec<String>>,
        /// regex conditions to filter in servers
        /// format: `<actor>:<value>,<actor>:<value>`
        /// supported actors: `PlayerName, PlayerUuid, Version, Protocol, ConnectedPlayers, MaxPlayers, Description, Favicon (base64 encoded)`
        #[clap(long, value_parser)]
        include_regex: Option<Vec<String>>,
        /// conditions to filter in servers
        /// format: `<actor>:<value>,<actor>:<value>`
        /// supported actors: `PlayerName, PlayerUuid, Version, Protocol, ConnectedPlayers, MaxPlayers, Description, Favicon (base64 encoded)`
        #[clap(long, value_parser)]
        include: Option<Vec<String>>,
    },
    /// Use to continually monitor for a condition of a minecraft server (e.g. a player logging on)
    Monitor {
        /// JSON file outputed by `masscan` or `ccheck scan`
        #[clap(value_parser)]
        input: PathBuf,

        /// Number of async tasks to scan with.
        #[clap(short, long, value_parser, default_value_t = 30)]
        workers: usize,
        /// Timeout for each server in milliseconds
        #[clap(short, long, value_parser, default_value_t = 1000)]
        timeout: u64,
        /// Webhook url to send alerts if server matching conditions is found
        #[clap(short, long, value_parser)]
        webhook_url: Option<String>,
        /// Dont exit if server matching conditions is found
        #[clap(long, value_parser, default_value_t = false)]
        dont_exit_on_success: bool,
        /// conditions to filter out servers
        /// format: `<actor>:<value>,<actor>:<value>`
        /// supported actors: `PlayerName, PlayerUuid, Version, Protocol, ConnectedPlayers, MaxPlayers, Description, Favicon (base64 encoded)`
        #[clap(long, value_parser)]
        exclude: Option<Vec<String>>,
        /// regex conditions to filter out servers
        /// format: `<actor>:<value>,<actor>:<value>`
        /// supported actors: `PlayerName, PlayerUuid, Version, Protocol, ConnectedPlayers, MaxPlayers, Description, Favicon (base64 encoded)`
        #[clap(long, value_parser)]
        exclude_regex: Option<Vec<String>>,
        /// regex conditions to filter in servers
        /// format: `<actor>:<value>,<actor>:<value>`
        /// supported actors: `PlayerName, PlayerUuid, Version, Protocol, ConnectedPlayers, MaxPlayers, Description, Favicon (base64 encoded)`
        #[clap(long, value_parser)]
        include_regex: Option<Vec<String>>,
        /// conditions to filter in servers
        /// format: `<actor>:<value>,<actor>:<value>`
        /// supported actors: `PlayerName, PlayerUuid, Version, Protocol, ConnectedPlayers, MaxPlayers, Description, Favicon (base64 encoded)`
        #[clap(long, value_parser)]
        include: Option<Vec<String>>,
    },
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.command {
        Some(command) => match command {
            Command::Monitor {
                input,
                workers,
                timeout,
                exclude,
                exclude_regex,
                include_regex,
                include,
                webhook_url,
                dont_exit_on_success,
            } => {
                let mut conds = vec![];
                for i in exclude.unwrap_or_default() {
                    let vals = i.split(':').collect::<Vec<&str>>();
                    conds.push(Condition {
                        values: vec![vals[1].to_string()],
                        values_regex: vec![],
                        conditon_type: ConditionType::Exclude,
                        actor: Actor::from_str(vals[0])?,
                    });
                }
                for i in include.unwrap_or_default() {
                    let vals = i.split(':').collect::<Vec<&str>>();
                    conds.push(Condition {
                        values: vec![vals[1].to_string()],
                        values_regex: vec![],
                        conditon_type: ConditionType::Include,
                        actor: Actor::from_str(vals[0])?,
                    });
                }
                for i in exclude_regex.unwrap_or_default() {
                    let vals = i.split(':').collect::<Vec<&str>>();
                    conds.push(Condition {
                        values: vec![],
                        values_regex: vec![Regex::new(vals[1])?],
                        conditon_type: ConditionType::Exclude,
                        actor: Actor::from_str(vals[0])?,
                    });
                }
                for i in include_regex.unwrap_or_default() {
                    let vals = i.split(':').collect::<Vec<&str>>();
                    conds.push(Condition {
                        values: vec![],
                        values_regex: vec![Regex::new(vals[1])?],
                        conditon_type: ConditionType::Include,
                        actor: Actor::from_str(vals[0])?,
                    });
                }
                let conds = Conditions { conditions: conds };
                let cnf = Config {
                    addrs: {
                        if let Ok(mformat) = MasscanFormat::try_from(
                            File::open(input.clone()).expect("invalid input file"),
                        ) {
                            mformat.get_ips()
                        } else {
                            CCheckFormat::try_from(File::open(input).expect("invalid input file"))
                                .expect("input file is of invalid format")
                                .servers
                                .iter()
                                .map(|sv| sv.ip)
                                .collect()
                        }
                    },
                    mode: Mode::Monitor {
                        workers,
                        webhook_url,
                        exit_on_success: !dont_exit_on_success,
                    },
                    conditions: conds,
                    timeout: Duration::from_millis(timeout),
                };
                cnf.run().await?;
            }
            Command::Scan {
                timeout,
                input,
                workers,
                output,
                exclude,
                exclude_regex,
                include_regex,
                include,
                progress_bar,
            } => {
                let mut conds = vec![];
                for i in exclude.unwrap_or_default() {
                    let vals = i.split(':').collect::<Vec<&str>>();
                    conds.push(Condition {
                        values: vec![vals[1].to_string()],
                        values_regex: vec![],
                        conditon_type: ConditionType::Exclude,
                        actor: Actor::from_str(vals[0])?,
                    });
                }
                for i in include.unwrap_or_default() {
                    let vals = i.split(':').collect::<Vec<&str>>();
                    conds.push(Condition {
                        values: vec![vals[1].to_string()],
                        values_regex: vec![],
                        conditon_type: ConditionType::Include,
                        actor: Actor::from_str(vals[0])?,
                    });
                }
                for i in exclude_regex.unwrap_or_default() {
                    let vals = i.split(':').collect::<Vec<&str>>();
                    conds.push(Condition {
                        values: vec![],
                        values_regex: vec![Regex::new(vals[1])?],
                        conditon_type: ConditionType::Exclude,
                        actor: Actor::from_str(vals[0])?,
                    });
                }
                for i in include_regex.unwrap_or_default() {
                    let vals = i.split(':').collect::<Vec<&str>>();
                    conds.push(Condition {
                        values: vec![],
                        values_regex: vec![Regex::new(vals[1])?],
                        conditon_type: ConditionType::Include,
                        actor: Actor::from_str(vals[0])?,
                    });
                }
                let conds = Conditions { conditions: conds };
                let format = Config {
                    timeout: Duration::from_millis(timeout),
                    mode: Mode::Scanner {
                        workers,
                        output,
                        progress_bar,
                    },
                    addrs: MasscanFormat::try_from(
                        File::open(input).expect("invalid masscan file"),
                    )?
                    .get_ips(),
                    conditions: conds,
                };
                format.run().await?;
            }
        },
        None => {
            bail!("You must specify a valid subcommand. Run with --help parameter for more information.");
        }
    }
    Ok(())
}
