/*
 *  secrets.rs
 *  (c) 2025 Teodor Potancok
 *
 *  This Source Code Form is subject to the terms of the Mozilla Public
 *  License, v. 2.0. If a copy of the MPL was not distributed with this
 *  file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use rand::distr::Alphanumeric;
use rand::prelude::*;

pub fn generate_secret() -> String {
    let rng = rand::rng();

    rng.sample_iter(&Alphanumeric)
        .take(16)
        .map(|c| c as char)
        .collect::<String>()
}
