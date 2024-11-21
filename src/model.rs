use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ConnectArgs<'a> {
    pub address: &'a str,
    pub port: i32
}

#[derive(Serialize, Deserialize)]
pub struct ConnectResponse {
    pub success: Option<bool>,
    pub error: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct TransmitArgs<'a> {
    pub message: &'a str
}

#[derive(Serialize, Deserialize)]
pub struct TransmitResponse {
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ConfirmArgs<'a> {
    pub msg: &'a str,
}

// For any single-argument commands (get, delete)
#[derive(Serialize, Deserialize)]
pub struct StoreKeyArg<'a> {
    pub path: &'a str,
    pub key: &'a str,
}

#[derive(Serialize, Deserialize)]
pub struct StoreSetArgs<'a> {
    pub path: &'a str,
    pub key: &'a str,
    pub value: &'a str,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct LoadOptions {
    pub autoSave: Option<bool>
}
#[derive(Serialize, Deserialize)]
pub struct Load<'a> {
    pub path: &'a str,
    pub options: Option<LoadOptions>
}
#[derive(Serialize, Deserialize)]
pub struct EntriesArgs {
    pub rid: f64
}

#[derive(Serialize, Deserialize)]
pub struct StorePluginSetArgs<'a> {
    pub rid: f64,
    pub key: &'a str,
    pub value: &'a str,
}

#[derive(Serialize, Deserialize)]
pub struct StorePluginGetArgs<'a> {
    pub rid: f64,
    pub key: &'a str,
}

#[derive(Serialize, Deserialize)]
pub struct StorePluginDeleteArgs<'a> {
    pub rid: f64,
    pub key: &'a str,
}
