use std::io::{self, BufRead, Read};

use log::{error, info};
use serde_json::{json, Value as Json};

use super::document_sync::DidOpenTextDocumentParams;
use super::lifecycle::{InitializeResult, ServerCapabilities, ServerInfo};

use crate::parser::{parser::Parser, parser::TreeSitterParser};

pub struct LspHandler {
    parser: Box<dyn Parser>,
}

impl LspHandler {
    /// Handles a JSON-RPC message.
    pub fn initialize() -> Result<Self, Box<dyn std::error::Error>> {
        let parser: Box<dyn Parser> = Box::new(TreeSitterParser::new()?);
        Ok(Self { parser })
    }

    pub fn handle_response(
        &mut self,
        method: String,
        params: Json,
    ) -> Result<Option<Json>, String> {
        info!("Received method: {:?}", method);
        match method.as_str() {
            "initialize" => {
                let result = self.handle_initialize();
                Ok(Some(json!(result)))
            }
            "initialized" => Ok(None),
            "shutdown" => Ok(None),
            "exit" => Ok(None),
            "textDocument/didOpen" => match self.handle_open_document(params) {
                Ok(_) => {
                    info!("Opened document");
                    Ok(None)
                }
                Err(e) => {
                    error!("Failed to open document: {}", e);
                    Err(e)
                }
            },
            "textDocument/didChange" => Ok(None),
            "textDocument/didClose" => Ok(None),
            _ => Err(format!("Unknown method: {}", method)),
        }
    }

    /// Handles the `initialize` request.
    pub fn handle_initialize(&mut self) -> InitializeResult {
        // Return the server capabilities
        // (textDocumentSync, hoverProvider, definitionProvider are default)
        InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(1),
                hover_provider: Some(true),
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
        params: Json,
    ) -> Result<DidOpenTextDocumentParams, String> {
        // Deserialize the params
        let params: DidOpenTextDocumentParams = serde_json::from_value(params)
            .map_err(|e| format!("Failed to deserialize params: {}", e))?;

        // Parse the document
        let text = params.text_document.text.clone();
        let _tree = self
            .parser
            .parse(&text)
            .map_err(|e| format!("Failed to parse: {:?}", e))?;

        Ok(params)
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
