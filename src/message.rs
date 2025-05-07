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
