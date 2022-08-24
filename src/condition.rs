// CCheck: utility for scanning and probing minecraft servers
// Copyright (C) 202&&2 cleonyc

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
use regex::Regex;
use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

use crate::adapters::CCheckResponse;
#[derive(Debug, Clone)]

pub struct Conditions {
    pub conditions: Vec<Condition>,
}
impl Conditions {
    pub fn is_valid(&self, resp: CCheckResponse) -> bool {
        for cond in &self.conditions {
            if !cond.matches(resp.clone()) {
                return false;
            }
        }
        return true;
    }
}
#[derive(Debug, Clone)]

pub struct Condition {
    pub values: Vec<String>,
    pub values_regex: Vec<Regex>,
    pub conditon_type: ConditionType,
    pub actor: Actor,
}

impl Condition {
    /// Returns true if response is what we want to keep
    pub fn matches(&self, resp: CCheckResponse) -> bool {
        let is_match = match self.conditon_type {
            ConditionType::Exclude => true,
            ConditionType::Include => false,
        };
        match self.actor {
            Actor::Version => {
                if self.check_regex_match(&resp.version) {
                    return !is_match;
                }
                if self.values.contains(&resp.version) {
                    return !is_match;
                }
            }
            Actor::Protocol => {
                if self.check_regex_match(&format!("{}", &resp.protocol)) {
                    return !is_match;
                }
                if self.values.contains(&format!("{}", &resp.protocol)) {
                    return !is_match;
                }
            }
            Actor::ConnectedPlayers => {
                if self.check_regex_match(&format!("{}", &resp.online_players)) {
                    return !is_match;
                }
                if self.values.contains(&format!("{}", &resp.online_players)) {
                    return !is_match;
                }
            }
            Actor::MaxPlayers => {
                if self.check_regex_match(&format!("{}", &resp.max_players)) {
                    return !is_match;
                }
                if self.values.contains(&format!("{}", &resp.max_players)) {
                    return !is_match;
                }
            }
            Actor::Description => {
                if self.check_regex_match(&resp.description.text) {
                    return !is_match;
                }
                if self.values.contains(&resp.description.text) {
                    return !is_match;
                }
            }
            Actor::Favicon => {
                if self
                    .values
                    .contains(&base64::encode(resp.favicon.unwrap_or(b"".to_vec())))
                {
                    return !is_match;
                }
            }
            Actor::PlayerName => {
                for p in &resp.sample.unwrap_or(vec!()) {
                    if self.check_regex_match(&p.name) {
                        return !is_match;
                    }
                    if self.values.contains(&p.name) {
                        return !is_match;
                    }
                }
            },
            Actor::PlayerUuid => {
                for p in &resp.sample.unwrap_or(vec!()) {
                    if self.check_regex_match(&p.id) {
                        return !is_match;
                    }
                    if self.values.contains(&p.id) {
                        return !is_match;
                    }
                }
            },
        }
        return is_match;
    }
    fn check_regex_match(&self, str: &String) -> bool {
        self.values_regex
            .iter()
            .find(|&r| r.is_match(str))
            .is_some()
    }
}
#[derive(Debug, Clone, Deserialize, Serialize)]

pub enum ConditionType {
    Exclude,
    Include,
}
#[derive(Debug, Clone, Deserialize, Serialize, EnumString)]

pub enum Actor {
    #[strum(ascii_case_insensitive)]
    Version,
    #[strum(ascii_case_insensitive)]
    Protocol,
    #[strum(ascii_case_insensitive)]
    PlayerName,
    #[strum(ascii_case_insensitive)]
    PlayerUuid,
    #[strum(ascii_case_insensitive)]
    ConnectedPlayers,
    #[strum(ascii_case_insensitive)]
    MaxPlayers,
    #[strum(ascii_case_insensitive)]
    Description,
    #[strum(ascii_case_insensitive)]
    Favicon,
}
