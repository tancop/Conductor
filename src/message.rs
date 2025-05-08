/*
 *  message.rs
 *  (c) 2025 Teodor Potancok
 *
 *  This Source Code Form is subject to the terms of the Mozilla Public
 *  License, v. 2.0. If a copy of the MPL was not distributed with this
 *  file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RpcRequest<'a> {
    pub command: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<u32>,
    pub args: serde_json::Value,
}
