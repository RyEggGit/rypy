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
                let response = match decoded["method"].as_str() {
                    Some("initialize") => lsp_handler.handle_initialize(),
                    _ => lsp::InitializeResult {
                        capabilities: lsp::ServerCapabilities {},
                        server_info: None,
                    },
                };
                let encoded = rpc::encode(serde_json::to_value(response).unwrap());
                println!("{}", encoded);
            }
            // todo handle error
            Err(e) => {}
        }
    }
}
