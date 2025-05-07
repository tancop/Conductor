use rand::distr::Alphanumeric;
use rand::prelude::*;

pub fn generate_secret() -> String {
    let rng = rand::rng();

    rng.sample_iter(&Alphanumeric)
        .take(16)
        .map(|c| c as char)
        .collect::<String>()
}
