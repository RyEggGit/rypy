use log::{debug, error};
use std::fs::File;
use std::io::Write;

mod lsp;
mod rpc;

fn main() {
    // Initialize the logger to save to file
    simplelog::CombinedLogger::init(vec![simplelog::WriteLogger::new(
        simplelog::LevelFilter::Debug,
        simplelog::Config::default(),
        File::create("lsp.log").unwrap(),
    )])
    .unwrap();

    // Create a new LSP handler
    let mut lsp_handler = lsp::handler::LspHandler::initialize();
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
                    Ok(result) => match result {
                        Some(result) => result,
                        None => continue,
                    },
                    Err(error) => { 
                        error!("Error: {:?}", error);
                        continue;
                    }
                };

                let encoded = rpc::encode_response(jsonrpc, id, result);
                debug!("Sending message: {:?}", encoded);
                // Add the Content-Length header
                let content_length = encoded.len();
                let message = format!("Content-Length: {}\r\n\r\n{}", content_length, encoded);
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
