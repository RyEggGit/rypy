use std::io::{self, Write};

mod lsp;
mod rpc;
use log::{debug, error, info, warn};
use simplelog::*;
use std::fs::File;

fn main() {
    // Initialize the logger to save to file
    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Debug,
        Config::default(),
        File::create("lsp.log").unwrap(),
    )])
    .unwrap();

    // Create a new LSP handler
    let mut lsp_handler = lsp::handler::LspHandler::new();
    loop {
        match lsp_handler.read_message() {
            Ok(message) => {
                debug!("Received message: {:?}", message);

                let (jsonrpc, id, method, params) = match rpc::decode_request(message) {
                    Ok(decoded) => decoded,
                    Err(e) => {
                        error!("Error decoding message: {:?}", e);
                        continue;
                    }
                };

                let result = match lsp_handler.handle_response(method, params) {
                    lsp::handler::LspResult::Success(result) => result,
                    lsp::handler::LspResult::Warning(warning) => {
                        warn!("Warning: {:?}", warning);
                        continue;
                    }
                    lsp::handler::LspResult::Error(error) => {
                        error!("Error: {:?}", error);
                        continue;
                    }
                };

                let encoded = rpc::encode_response(jsonrpc, id, result);
                debug!("Sending message: {:?}", encoded);
                // Add the Content-Length header
                let content_length = encoded.len();
                let message = format!("Content-Length: {}\r\n\r\n{}", content_length, encoded);
                io::stdout().write_all(message.as_bytes()).unwrap();
                io::stdout().flush().unwrap();
            }
            Err(e) => {
                error!("Error reading message: {:?}", e);
                break;
            }
        }
    }
}
