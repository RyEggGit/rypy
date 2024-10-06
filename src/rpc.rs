use serde_json::Value;

pub fn encode(value: Value) -> String {
    value.to_string()
}

pub fn decode(message: String) -> Value {
    serde_json::from_str(&message).unwrap()
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use crate::rpc;
    

    #[test]
    fn test_encode_decode() {
        let original = json!({
            "method": "initialize",
            "params": {
                "capabilities": {}
            }
        });

        let encoded = rpc::encode(original.clone());
        let decoded = rpc::decode(encoded);

        assert_eq!(original, decoded);
    }
}
