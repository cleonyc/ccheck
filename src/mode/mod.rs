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
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub mod monitor;
pub mod scanner;
#[derive(Serialize, Deserialize, Clone)]
pub enum Mode {
    Scanner {
        workers: usize,
        output: PathBuf,
        progress_bar: bool,
    },
    Monitor {
        workers: usize,
        webhook_url: Option<String>,
        exit_on_success: bool,
    },
}
