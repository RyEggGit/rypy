use log::{debug, error};
use simplelog::{CombinedLogger, ConfigBuilder, WriteLogger, LevelFilter};
use std::fs::File;
use std::io::Write;

mod lsp;
mod parser;
mod rpc;
mod analysis;
mod storage;

fn main() {
    // Initialize the logger to save to file
    let mut config = ConfigBuilder::new();
    config.set_target_level(LevelFilter::Error); // Prevents verbose target logs
    config.add_filter_ignore_str("salsa"); // Ignore salsa logs

    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Debug,
        config.build(),
        File::create("lsp.log").unwrap(),
    )])
    .unwrap();

    


    // Create a new LSP handler
    let mut lsp_handler = match lsp::handler::LspHandler::initialize() {
        Ok(lsp_handler) => lsp_handler,
        Err(error) => {
            error!("Error: {:?}", error);
            panic!();
        }
    };

    loop {
        // Read a message from stdin
        match lsp_handler.read_message() {
            Ok(message) => {
                debug!("Received message: {:?}", message);

                // Decode the message
                let (jsonrpc, id, method, params) = match rpc::decode_request(message) {
                    Ok(decoded) => decoded,
                    Err(e) => {
                        error!("Error decoding message: {:?}", e);
                        continue;
                    }
                };

                // Handle the message
                let result = match lsp_handler.handle_response(method, params) {
                    Ok(result) => match result {
                        Some(result) => result,
                        None => continue,
                    },
                    Err(error) => {
                        error!("Error: {:?}", error);
                        continue;
                    }
                };

                // Encode the response
                let encoded = rpc::encode_response(jsonrpc, id, result);
                debug!("Sending message: {:?}", encoded);
                // Add the Content-Length header
                let content_length = encoded.len();
                let message = format!("Content-Length: {}\r\n\r\n{}", content_length, encoded);

                // Write the response to stdout
                std::io::stdout().write_all(message.as_bytes()).unwrap();
                std::io::stdout().flush().unwrap();
            }
            Err(e) => {
                error!("Error reading message: {:?}", e);
                break;
            }
        }
    }
}
