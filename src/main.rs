mod log;
mod lsp;
mod rpc;

fn main() {
    let mut logger = log::Logger::new("lsp.log").unwrap();
    let mut lsp_handler = lsp::LspHandler::new(&mut logger);

    loop {
        match lsp_handler.read_message() {
            Ok(message) => {
                let decoded = rpc::decode(message);
                let response = match lsp_handler.handle_response(decoded) {
                    lsp::LspResult::Success(Some(response)) => response,
                    lsp::LspResult::Success(None) => {
                        // No response to send (no point in handling this case)
                        continue;
                    }
                    lsp::LspResult::Error(_) => {
                        // TODO: improve error handling. Perhaps log tracing to the client?
                        continue;
                    }
                };

                let encoded = rpc::encode(response);
                println!("{}", encoded);
            }
            // TODO: improve error handling. Perhaps log tracing to the client?
            Err(_) => {}
        }
    }
}
