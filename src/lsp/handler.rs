use std::io::{self, BufRead, Read};

use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};

use crate::lsp::lifecycle::{ServerCapabilities, ServerInfo};

use super::lifecycle::InitializeResult;

#[derive(Serialize, Deserialize, Debug)]
pub enum LspResult {
    Success(Json),
    Warning(String),
    Error(String),
}
pub struct LspHandler {}

impl LspHandler {
    pub fn new() -> LspHandler {
        return LspHandler {};
    }

    pub fn handle_initialize(&mut self) -> InitializeResult {
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

    pub fn handle_response(&mut self, method: String, _: Json) -> LspResult {
        debug!("Received method: {:?}", method);
        let response = match method.as_str() {
            "initialize" => {
                let result = self.handle_initialize();
                json!(result)
            }
            "initialized" => {
                return LspResult::Warning(
                    "Client was initalized. Using this as no response".to_string(),
                );
            }
            "shutdown" => {
                json!(null)
            }
            "exit" => {
                json!(null)
            }
            "textDocument/didOpen" => {
                json!({"result": null})
            }
            "textDocument/didChange" => {
                json!({"result": null})
            }
            "textDocument/didClose" => {
                json!({"result": null})
            }
            _ => {
                return LspResult::Error("Method not found".to_string());
            }
        };

        return LspResult::Success(response);
    }

    pub fn read_message(&mut self) -> io::Result<String> {
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        let mut buffer = String::new();
        let mut content_length = None;

        // Read headers
        loop {
            buffer.clear();
            handle.read_line(&mut buffer)?;

            // Check for the end of headers
            if buffer.trim().is_empty() {
                break;
            }

            // Parse headers
            if let Some((key, value)) = buffer.split_once(": ") {
                if key == "Content-Length" {
                    content_length = Some(value.trim().parse::<usize>().map_err(|e| {
                        error!("Invalid Content-Length: {}", e);
                        io::Error::new(io::ErrorKind::InvalidData, "Invalid Content-Length")
                    })?);
                }
            } else {
                error!("Malformed header: {}", buffer);
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Malformed header",
                ));
            }
        }

        // Ensure Content-Length is found
        let content_length = content_length.ok_or_else(|| {
            error!("Missing Content-Length header");
            io::Error::new(io::ErrorKind::InvalidData, "Missing Content-Length header")
        })?;

        // Read the body
        let mut body = vec![0; content_length];
        handle.read_exact(&mut body)?;

        // Convert body to UTF-8 string
        String::from_utf8(body).map_err(|e| {
            error!("Invalid UTF-8 body: {}", e);
            io::Error::new(io::ErrorKind::InvalidData, format!("Invalid UTF-8: {}", e))
        })
    }
}
