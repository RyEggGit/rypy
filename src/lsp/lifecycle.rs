use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

#[derive(Serialize, Deserialize, Debug)]
pub struct InitializeParams {
    pub capabilities: ClientCapabilities,
    pub process_id: Option<u32>,
    pub root_uri: Option<String>,
    pub initialization_options: Option<Json>,
    pub trace: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientCapabilities {
    // Define client capabilities here
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InitializeResult {
    pub capabilities: ServerCapabilities,
    pub server_info: Option<ServerInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ServerCapabilities {
    pub text_document_sync: Option<u8>,
    pub hover_provider: Option<bool>,
    pub definition_provider: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerInfo {
    pub name: String,
    pub version: Option<String>,
}
