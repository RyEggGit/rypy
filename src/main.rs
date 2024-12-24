use std::{io::{self, Write}, result};

mod lsp;
mod rpc;
use log::info;
use serde_json::to_string;
use simplelog::*;
use std::fs::File;

fn main() {
    // Initialize the logger to save to file
    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Info,
        Config::default(),
        File::create("lsp.log").unwrap(),
    )])
    .unwrap();

    // Create a new LSP handler
    let mut lsp_handler = lsp::LspHandler::new();
    loop {
        match lsp_handler.read_message() {
            Ok(message) => {
                info!("Received message: {:?}", message);

                let (jsonrpc, id, method, params) = rpc::decode(message);
                let result = match lsp_handler.handle_response(method, params) {
                    lsp::LspResult::Success(result) => result,
                    lsp::LspResult::Error(_) => continue,
                };

                let encoded = rpc::encode(jsonrpc, id, result);
                info!("Sending message: {:?}", encoded);
                // Add the Content-Length header
                let content_length = encoded.len();
                let header = format!("Content-Length: {}\r\n\r\n", content_length);
                io::stdout().write_all(header.as_bytes()).unwrap();
                io::stdout().write_all(encoded.as_bytes()).unwrap();
                io::stdout().flush().unwrap();
            }
            Err(_) => {}
        }
    }
}
