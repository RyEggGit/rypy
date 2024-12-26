use serde_json::Value as Json;

pub fn encode_request(jsonrpc: String, id: Option<i64>, method: String, params: Json) -> String {
    let message = serde_json::json!({
        "jsonrpc": jsonrpc,
        "id": id,
        "method": method,
        "params": params
    });
    message.to_string()
}

pub fn encode_response(jsonrpc: String, id: Option<i64>, result: Json) -> String {
    let message = serde_json::json!({
        "jsonrpc": jsonrpc,
        "id": id,
        "result": result
    });
    message.to_string()
}

pub fn decode_request(message: String) -> Result<(String, Option<i64>, String, Json), String> {
    let value: Json =
        serde_json::from_str(&message).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let jsonrpc = value["jsonrpc"]
        .as_str()
        .ok_or_else(|| "Missing or invalid 'jsonrpc' field".to_string())?
        .to_string();

    let id = value["id"].as_i64();

    let method = value["method"]
        .as_str()
        .ok_or_else(|| "Missing or invalid 'method' field".to_string())?
        .to_string();

    let params = value["params"].clone();

    Ok((jsonrpc, id, method, params))
}

#[cfg(test)]
mod main_tests {
    use crate::rpc;
    use serde_json::json;

    #[test]
    fn test_encode_decode() {
        let original = (
            "2.0".to_string(),
            Some(1),
            "textDocument/didOpen".to_string(),
            json!({
                "textDocument": {
                    "uri": "file:///path/to/file",
                    "languageId": "rust",
                    "version": 1,
                    "text": "fn main() {}"
                }
            }),
        );

        let encoded = rpc::encode_request(
            original.0.clone(),
            original.1,
            original.2.clone(),
            original.3.clone(),
        );
        let decoded = rpc::decode_request(encoded).unwrap();
        assert_eq!(decoded.0, original.0);
        assert_eq!(decoded.1, original.1);
        assert_eq!(decoded.2, original.2);
        assert_eq!(decoded.3, original.3);
    }

    #[test]
    fn test_encode_response() {
        let response = rpc::encode_response("2.0".to_string(), Some(1), json!({ "success": true }));
        let parsed: serde_json::Value = serde_json::from_str(&response).unwrap();
        assert_eq!(parsed["result"]["success"], json!(true));
    }
}
