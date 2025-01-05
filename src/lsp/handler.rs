use std::io::{self, BufRead, Read};

use log::{error, info, warn};
use serde_json::{json, Value as Json};

use super::document_sync::DidOpenTextDocumentParams;
use super::language_features::Location;
use super::lifecycle::{InitializeResult, ServerCapabilities, ServerInfo};

use crate::lsp::language_features::GotoDefinitionParams;
use crate::storage::state::LspState;

pub struct LspHandler {
    shutdown: bool,
    state: LspState,
}

impl LspHandler {
    /// Handles a JSON-RPC message.
    pub fn initialize() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            shutdown: false,
            state: LspState::new(),
        })
    }

    pub fn handle_response(
        &mut self,
        method: String,
        params: Json,
    ) -> Result<Option<Json>, String> {
        info!("Received method: {:?}", method);
        // Check if client requested shutdown
        if self.shutdown && method != "exit" {
            return Err("Server is shutting down".to_string());
        }
        match method.as_str() {
            "initialize" => {
                let result = self.handle_initialize();
                Ok(Some(json!(result)))
            }
            "initialized" => Ok(None),
            "shutdown" => {
                self.handle_shutdown();
                warn!("Shutting down");
                Ok(Some(json!(null)))
            }
            "exit" => {
                info!("Exiting");
                Ok(None)
            }
            "textDocument/didOpen" => {
                let params: DidOpenTextDocumentParams = serde_json::from_value(params)
                    .map_err(|e| format!("Failed to deserialize params: {}", e))?;
                let result = self.handle_open_document(params)?;
                Ok(Some(json!(result)))
            }
            "textDocument/didChange" => Ok(None),
            "textDocument/didClose" => Ok(None),
            "textDocument/definition" => {
                let params: GotoDefinitionParams = serde_json::from_value(params)
                    .map_err(|e| format!("Failed to deserialize params: {}", e))?;
                let result = self.handle_go_to_definition(params)?;
                Ok(Some(json!(result)))
            }
            _ => Err(format!("Unknown method: {}", method)),
        }
    }

    /// Handles the `initialize` request.
    pub fn handle_initialize(&mut self) -> InitializeResult {
        // Return the server capabilities and info
        InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(1),
                hover_provider: Some(false),
                definition_provider: Some(true),
            },
            server_info: Some(ServerInfo {
                name: "rypy 🐍".to_string(),
                version: Some("0.0.0".to_string()),
            }),
        }
    }

    /// Handles the `textDocument/didOpen` notification.
    pub fn handle_open_document(
        &mut self,
        params: DidOpenTextDocumentParams,
    ) -> Result<(), String> {
        self.state.open_document(params);
        Ok(())
    }

    /// Handles the `textDocument/definition` request.
    pub fn handle_go_to_definition(
        &mut self,
        params: GotoDefinitionParams,
    ) -> Result<Location, String> {
        match self.state.get_definition(params) {
            Some(location) => Ok(location),
            None => Err("Definition not found".to_string()),
        }
    }

    /// Handles the `shutdown` notification.
    pub fn handle_shutdown(&mut self) {
        self.shutdown = true;
    }

    /// Extracts the JSON-RPC message from stdin.
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
