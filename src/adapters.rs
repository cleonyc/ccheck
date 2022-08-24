// **IMPORTANT COPYRIGHT DISCLAIMER**
// THIS CODE IS MODIFIED FROM MIT LICENSED CODE
// The original copyright disclaimer is listed below, until line 10.
// This only applies to the original code. Not to any modifications,
// the whole file, and/or this whole program.
// MIT License

// Copyright (c) 2019 kiwiyou

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

// All modifications, and this whole file modified should still be considered
// licensed under the terms shown below:
//
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

use craftping::{Chat, Player, Response};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
/// A ping response returned from server.
pub struct CCheckResponse {
    
    /// The version name of the server.
    pub version: String,
    /// The protocol number of the server.
    /// See also [the minecraft protocol wiki](https://wiki.vg/Protocol_version_numbers) for the actual values.
    pub protocol: i32,
    /// The maximum number of the connected players.
    pub max_players: usize,
    /// The number of the players currently connected.
    pub online_players: usize,
    /// The sample of the connected players.
    /// Note that it can be `None` even if some players are connected.
    pub sample: Option<Vec<CCheckPlayer>>,
    /// The description (aka MOTD) of the server.
    /// See also [the minecraft protocol wiki](https://wiki.vg/Chat#Current_system_.28JSON_Chat.29) for the [`Chat`](Chat) format.
    pub description: CCheckChat,
    /// The favicon of the server in PNG format.
    pub favicon: Option<Vec<u8>>,
    // Disabled for now. Not currently interested in adding support for checking for forge protocol info

    // The mod information object used in FML protocol (version 1.7 - 1.12).
    // See also [the minecraft protocol wiki](https://wiki.vg/Minecraft_Forge_Handshake#FML_protocol_.281.7_-_1.12.29)
    // for the [`ModInfo`](ModInfo) format.
    // pub mod_info: Option<CCheckModInfo>,

    //The forge information object used in FML2 protocol (version 1.13 - current).
    // See also [the minecraft protocol wiki](https://wiki.vg/Minecraft_Forge_Handshake#FML2_protocol_.281.13_-_Current.29)
    // for the [`ForgeData`](ForgeData) format.
    // pub forge_data: Option<CCheckForgeData>,
}
impl From<Response> for CCheckResponse {
    fn from(res: Response) -> Self {
        let mut new_sample: Vec<CCheckPlayer> = vec![];
        if let Some(sample) = res.sample {
            for player in sample {
                new_sample.push(player.into())
            }
        }

        return CCheckResponse {
            version: res.version,
            protocol: res.protocol,
            max_players: res.max_players,
            online_players: res.online_players,
            sample: {
                if new_sample.len() != 0 {
                    Some(new_sample)
                } else {
                    None
                }
            },
            description: res.description.into(),
            favicon: res.favicon,
        };
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// The sample players' information.
pub struct CCheckPlayer {
    /// The name of the player.
    pub name: String,
    /// The uuid of the player.
    /// Normally used to identify a player.
    pub id: String,
}
impl From<Player> for CCheckPlayer {
    fn from(p: Player) -> Self {
        return Self {
            name: p.name,
            id: p.id,
        };
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
/// The chat component used in the server description.
///
/// See also [the minecraft protocol wiki](https://wiki.vg/Chat#Current_system_.28JSON_Chat.29).
pub struct CCheckChat {
    /// The text which this `Chat` object holds.
    pub text: String,
    #[serde(default)]
    /// `true` if the text *and* the extras should be __bold__.
    pub bold: bool,
    #[serde(default)]
    /// `true` if the text *and* the extras should be *italic*.
    pub italic: bool,
    #[serde(default)]
    /// `true` if the text *and* the extras should be <u>underlined</u>.
    pub underlined: bool,
    #[serde(default)]
    /// `true` if the text *and* the extras should have a <strike>strikethrough</strike>.
    pub strikethrough: bool,
    #[serde(default)]
    /// `true` if the text *and* the extras should look obfuscated.
    pub obfuscated: bool,
    /// The color which the text and the extras should have.
    /// `None` to use default color.
    pub color: Option<String>,
    #[serde(default)]
    /// The extra text components following this text.
    /// They should inherit this chat component's properties (bold, italic, etc.) but can also override the properties.
    pub extra: Vec<CCheckChat>,
}
impl From<Chat> for CCheckChat {
    fn from(chat: Chat) -> Self {
        let mut new_extra: Vec<CCheckChat> = vec![];
        for chat in chat.extra {
            new_extra.push(chat.into())
        }
        Self {
            text: chat.text,
            bold: chat.bold,
            italic: chat.italic,
            underlined: chat.underlined,
            strikethrough: chat.strikethrough,
            obfuscated: chat.obfuscated,
            color: chat.color,
            extra: new_extra,
        }
    }
}
