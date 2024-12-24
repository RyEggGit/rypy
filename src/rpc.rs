use serde_json::Value as Json;

pub fn encode(jsonrpc: String, id: i64, result: Json) -> String {
    let message = serde_json::json!({
        "jsonrpc": jsonrpc,
        "id": id,
        "result": result
    });
    message.to_string()
}
pub fn decode(message: String) -> (String, i64, String, Json) {
    let value: Json = serde_json::from_str(&message).unwrap();
    let jsonrpc = value["jsonrpc"].as_str().unwrap().to_string();
    let id = value["id"].as_i64().unwrap();
    let method = value["method"].as_str().unwrap().to_string();
    let params = value["params"].clone();
    (jsonrpc, id, method, params)
}

#[cfg(test)]
mod tests {
    use crate::rpc;
    use serde_json::json;

    #[test]
    fn test_encode_decode() {
        let original = (
            "2.0".to_string(),
            1,
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

        let encoded = rpc::encode(original.0.clone(), original.1, original.3.clone());
        let decoded = rpc::decode(encoded);
        assert_eq!(original, decoded);
    }
}
