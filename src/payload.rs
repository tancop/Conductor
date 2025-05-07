use regex::{Captures, Regex};

pub fn make_payload(template: &str, port: u16, replace: bool, secret: String) -> String {
    let re = Regex::new(r"(\$REPLACE)|(\$SECRET)|(\$PORT)").unwrap();

    let payload = re
        .replace_all(template, |caps: &Captures| match &caps[0] {
            "$REPLACE" => replace.to_string(),
            "$SECRET" => secret.to_string(),
            "$PORT" => port.to_string(),
            _ => unreachable!(),
        })
        .to_string();

    payload
}
