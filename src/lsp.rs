use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};
use std::io::{self, BufRead, Read};

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
    text_document_sync: Option<u8>,
    hover_provider: Option<bool>,
    definition_provider: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerInfo {
    pub name: String,
    pub version: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum LspResult {
    Success(Json),
    Error(String),
}

pub struct LspHandler {}

impl LspHandler {
    pub fn new() -> LspHandler {
        return LspHandler {};
    }

    pub fn handle_initialize(&mut self) -> InitializeResult {
        info!("Handling initialize request");

        InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(1),
                hover_provider: Some(true),
                definition_provider: Some(true),
            },
            server_info: Some(ServerInfo {
                name: "My Language Server".to_string(),
                version: Some("1.0.0".to_string()),
            }),
        }
    }

    pub fn handle_response(&mut self, method: String, params: Json) -> LspResult {
        let response = match method.as_str() {
            "initialize" => {
                let result = self.handle_initialize();
                json!(result)
            }
            "shutdown" => {
                self.handle_shutdown();
                json!({"result": null})
            }
            "textDocument/didOpen" => {
                self.handle_did_open(params);
                json!({"result": null})
            }
            "textDocument/didChange" => {
                self.handle_did_change(params);
                json!({"result": null})
            }
            "textDocument/didClose" => {
                self.handle_did_close(params);
                json!({"result": null})
            }
            _ => {
                return LspResult::Error("Method not found".to_string());
            }
        };

        return LspResult::Success(response);
    }

    pub fn handle_shutdown(&mut self) {
        info!("Handling shutdown request");
    }

    pub fn handle_did_open(&mut self, params: Json) {
        info!("Handling didOpen: {:?}", params);
    }

    pub fn handle_did_change(&mut self, params: Json) {
        info!("Handling didChange: {:?}", params);
    }

    pub fn handle_did_close(&mut self, params: Json) {
        info!("Handling didClose: {:?}", params);
    }

    pub fn read_message(&mut self) -> io::Result<String> {
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        let mut buffer = String::new();
        let mut content_length = 0;

        loop {
            buffer.clear();
            handle.read_line(&mut buffer)?;
            if buffer == "\r\n" {
                break;
            }

            let mut parts = buffer.split(": ");
            let key = parts.next().unwrap_or_default();
            let value = parts.next().unwrap_or_default();
            if key == "Content-Length" {
                content_length = match value.trim().parse() {
                    Ok(length) => length,
                    Err(e) => {
                        error!("Invalid Content-Length: {}", e);
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Invalid Content-Length",
                        ));
                    }
                };
            }
        }

        let mut body = vec![0; content_length];
        handle.read_exact(&mut body)?;
        let message =
            String::from_utf8(body).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(message)
    }
}
