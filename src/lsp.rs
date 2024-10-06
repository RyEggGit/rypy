use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};
use std::io::{self, BufRead, Read};

use crate::log::Logger;

#[derive(Serialize, Deserialize)]
pub struct InitializeParams {
    pub capabilities: ClientCapabilities,
    pub process_id: Option<u32>,
    pub root_uri: Option<String>,
    pub initialization_options: Option<Json>,
    pub trace: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ClientCapabilities {
    // Define client capabilities here
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InitializeResult {
    pub capabilities: ServerCapabilities,
    pub server_info: Option<ServerInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerCapabilities {}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerInfo {
    pub name: String,
    pub version: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum LspResult {
    Success(Option<Json>),
    Error(String),
}

pub struct LspHandler<'a> {
    logger: &'a mut Logger, // Add a logger field
}

impl<'a> LspHandler<'a> {
    pub fn new(logger: &mut Logger) -> LspHandler {
        LspHandler { logger }
    }

    pub fn handle_initialize(&mut self) -> InitializeResult {
        self.logger.log("Handling initialize request").unwrap();

        InitializeResult {
            capabilities: ServerCapabilities {
                // Initialize server capabilities here
            },
            server_info: Some(ServerInfo {
                name: "My Language Server".to_string(),
                version: Some("1.0.0".to_string()),
            }),
        }
    }

    pub fn handle_response(&mut self, message: Json) -> LspResult {
        self.logger
            .log(&format!("Handling response: {:?}", message))
            .unwrap();

        let response = match message["method"].as_str() {
            Some("initialize") => {
                let result = self.handle_initialize();
                json!({"result": result})
            }
            Some("shutdown") => {
                self.handle_shutdown();
                json!({"result": null})
            }
            Some("textDocument/didOpen") => {
                self.handle_did_open(message["params"].clone());
                json!({"result": null})
            }
            Some("textDocument/didChange") => {
                self.handle_did_change(message["params"].clone());
                json!({"result": null})
            }
            Some("textDocument/didClose") => {
                self.handle_did_close(message["params"].clone());
                json!({"result": null})
            }
            _ => {
                return LspResult::Error("Method not found".to_string());
            }
        };

        return LspResult::Success(Some(response));
    }

    pub fn handle_shutdown(&mut self) {
        self.logger.log("Handling shutdown request").unwrap();
    }

    pub fn handle_did_open(&mut self, params: Json) {
        self.logger
            .log(&format!("Handling didOpen: {:?}", params))
            .unwrap();
    }

    pub fn handle_did_change(&mut self, params: Json) {
        self.logger
            .log(&format!("Handling didChange: {:?}", params))
            .unwrap();
    }

    pub fn handle_did_close(&mut self, params: Json) {
        self.logger
            .log(&format!("Handling didClose: {:?}", params))
            .unwrap();
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
                        self.logger
                            .log(&format!("Error parsing Content-Length: {}", e))
                            .unwrap();
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
