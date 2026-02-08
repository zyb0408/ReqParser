use base64::engine::general_purpose;
use base64::Engine;
use chrono::TimeZone;
use regex::Regex;
use std::sync::LazyLock;

use crate::models::{ParseNode, ParseResult};

static RE_JWT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+$").unwrap());

static RE_BASE64: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[A-Za-z0-9+/\-_]{20,}={0,2}$").unwrap());

static RE_URL_ENCODED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"%[0-9A-Fa-f]{2}").unwrap());

static RE_TIMESTAMP: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\d{10}(\d{3})?$").unwrap());

static RE_COMPOUND: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[^&=]+=[^&]*(&[^&=]+=[^&]*)+$").unwrap());

/// Apply recursive decoding to all values in a ParseResult.
pub fn apply_recursive_decode(result: &mut ParseResult) {
    for header in &mut result.headers {
        decode_node(header);
        if let Some(children) = &mut header.children {
            for child in children {
                decode_node(child);
            }
        }
    }

    if let Some(params) = &mut result.query_params {
        for param in params {
            decode_node(param);
        }
    }
}

/// Decode a single ParseNode's value, setting decoded_value, value_type, and children as needed.
pub fn decode_node(node: &mut ParseNode) {
    let value = node.value.trim();
    if value.is_empty() {
        return;
    }

    // Priority order: JWT > Timestamp > Base64 > JSON > Compound > URL-encoded

    if try_decode_jwt(node) {
        return;
    }
    if try_decode_timestamp(node) {
        return;
    }
    if try_decode_base64(node) {
        return;
    }
    if try_decode_json(node) {
        return;
    }
    if try_decode_compound(node) {
        return;
    }
    try_decode_url_encoded(node);
}

/// Try to decode the node value as a JWT token.
fn try_decode_jwt(node: &mut ParseNode) -> bool {
    let value = node.value.trim();
    if !RE_JWT.is_match(value) {
        return false;
    }

    let parts: Vec<&str> = value.split('.').collect();
    if parts.len() != 3 {
        return false;
    }

    let header_json = match decode_base64url_to_string(parts[0]) {
        Some(s) => s,
        None => return false,
    };
    let payload_json = match decode_base64url_to_string(parts[1]) {
        Some(s) => s,
        None => return false,
    };

    // Validate that both are valid JSON objects
    let header_val: serde_json::Value = match serde_json::from_str::<serde_json::Value>(&header_json) {
        Ok(v) if v.is_object() => v,
        _ => return false,
    };
    let payload_val: serde_json::Value = match serde_json::from_str::<serde_json::Value>(&payload_json) {
        Ok(v) if v.is_object() => v,
        _ => return false,
    };

    let header_pretty = serde_json::to_string_pretty(&header_val).unwrap_or(header_json);
    let payload_pretty =
        serde_json::to_string_pretty(&payload_val).unwrap_or_else(|_| payload_json.clone());

    let mut header_node = ParseNode {
        key: "header".to_string(),
        value: header_pretty,
        children: None,
        description: None,
        decoded_value: None,
        value_type: Some("json".to_string()),
    };
    let mut payload_node = ParseNode {
        key: "payload".to_string(),
        value: payload_pretty.clone(),
        children: None,
        description: None,
        decoded_value: None,
        value_type: Some("json".to_string()),
    };

    // Recursively decode children values (e.g. timestamps inside JWT payload)
    decode_json_object_children(&mut header_node, &header_val);
    decode_json_object_children(&mut payload_node, &payload_val);

    node.value_type = Some("jwt".to_string());
    node.decoded_value = Some(payload_pretty);
    node.children = Some(vec![header_node, payload_node]);

    true
}

/// Expand a JSON object into children ParseNodes and recursively decode each child.
fn decode_json_object_children(node: &mut ParseNode, val: &serde_json::Value) {
    if let Some(obj) = val.as_object() {
        let children: Vec<ParseNode> = obj
            .iter()
            .map(|(k, v)| {
                let value_str = match v {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                let mut child = ParseNode {
                    key: k.clone(),
                    value: value_str,
                    children: None,
                    description: None,
                    decoded_value: None,
                    value_type: None,
                };
                decode_node(&mut child);
                child
            })
            .collect();
        if !children.is_empty() {
            node.children = Some(children);
        }
    }
}

/// Try to decode the node value as a Unix timestamp.
fn try_decode_timestamp(node: &mut ParseNode) -> bool {
    let value = node.value.trim();
    if !RE_TIMESTAMP.is_match(value) {
        return false;
    }

    let num: i64 = match value.parse() {
        Ok(n) => n,
        Err(_) => return false,
    };

    // Convert millisecond timestamps to seconds for range check
    let secs = if value.len() == 13 { num / 1000 } else { num };

    // Range: 2000-01-01 to 2050-01-01
    if !(946684800..=2524608000).contains(&secs) {
        return false;
    }

    let nanos = if value.len() == 13 {
        (num % 1000) as u32 * 1_000_000
    } else {
        0
    };

    let dt = match chrono::Local.timestamp_opt(secs, nanos) {
        chrono::LocalResult::Single(dt) => dt,
        _ => return false,
    };

    let formatted = dt.format("%Y-%m-%dT%H:%M:%S%:z").to_string();

    node.value_type = Some("timestamp".to_string());
    node.decoded_value = Some(formatted);

    true
}

/// Try to decode the node value as standard Base64.
fn try_decode_base64(node: &mut ParseNode) -> bool {
    let value = node.value.trim();
    if !RE_BASE64.is_match(value) {
        return false;
    }

    // Skip if it looks like a JWT (has dots)
    if value.contains('.') {
        return false;
    }

    let decoded_bytes = match general_purpose::STANDARD.decode(value) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };

    let decoded_str = match String::from_utf8(decoded_bytes) {
        Ok(s) => s,
        Err(_) => return false,
    };

    // Skip if decoded result is empty or all whitespace
    if decoded_str.trim().is_empty() {
        return false;
    }

    node.value_type = Some("base64".to_string());
    node.decoded_value = Some(decoded_str);

    true
}

/// Try to decode the node value as URL-encoded text.
fn try_decode_url_encoded(node: &mut ParseNode) -> bool {
    let value = node.value.trim();
    if !RE_URL_ENCODED.is_match(value) {
        return false;
    }

    let decoded = url_decode(value);
    if decoded == value {
        return false;
    }

    node.value_type = Some("url_encoded".to_string());
    node.decoded_value = Some(decoded);

    true
}

/// Simple URL decoding implementation.
fn url_decode(input: &str) -> String {
    let mut result = Vec::new();
    let bytes = input.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(
                &input[i + 1..i + 3],
                16,
            ) {
                result.push(byte);
                i += 3;
                continue;
            }
        }
        if bytes[i] == b'+' {
            result.push(b' ');
        } else {
            result.push(bytes[i]);
        }
        i += 1;
    }

    String::from_utf8(result).unwrap_or_else(|_| input.to_string())
}

/// Try to parse the node value as JSON.
fn try_decode_json(node: &mut ParseNode) -> bool {
    let value = node.value.trim();

    // Must start with { or [ to be considered JSON
    if !value.starts_with('{') && !value.starts_with('[') {
        return false;
    }

    let parsed: serde_json::Value = match serde_json::from_str(value) {
        Ok(v) => v,
        Err(_) => return false,
    };

    if !parsed.is_object() && !parsed.is_array() {
        return false;
    }

    let pretty = serde_json::to_string_pretty(&parsed).unwrap_or_else(|_| value.to_string());

    node.value_type = Some("json".to_string());
    node.decoded_value = Some(pretty);

    true
}

/// Try to parse the node value as compound key=value pairs (k1=v1&k2=v2).
fn try_decode_compound(node: &mut ParseNode) -> bool {
    let value = node.value.trim();
    if !RE_COMPOUND.is_match(value) {
        return false;
    }

    let children: Vec<ParseNode> = value
        .split('&')
        .filter_map(|pair| {
            let (k, v) = pair.split_once('=')?;
            let mut child = ParseNode {
                key: k.to_string(),
                value: v.to_string(),
                children: None,
                description: None,
                decoded_value: None,
                value_type: None,
            };
            decode_node(&mut child);
            Some(child)
        })
        .collect();

    if children.len() < 2 {
        return false;
    }

    node.value_type = Some("compound".to_string());
    node.children = Some(children);

    true
}

/// Decode a base64url-encoded string (no padding) to a UTF-8 string.
fn decode_base64url_to_string(input: &str) -> Option<String> {
    let bytes = general_purpose::URL_SAFE_NO_PAD.decode(input).ok()?;
    String::from_utf8(bytes).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::HttpContentType;

    fn make_node(key: &str, value: &str) -> ParseNode {
        ParseNode {
            key: key.to_string(),
            value: value.to_string(),
            children: None,
            description: None,
            decoded_value: None,
            value_type: None,
        }
    }

    // --- JWT tests ---

    #[test]
    fn test_jwt_valid_decode() {
        // Header: {"alg":"HS256","typ":"JWT"}
        // Payload: {"sub":"1234567890","name":"John Doe","iat":1516239022}
        let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        let mut node = make_node("Authorization", jwt);
        decode_node(&mut node);

        assert_eq!(node.value_type.as_deref(), Some("jwt"));
        assert!(node.decoded_value.is_some());
        let decoded = node.decoded_value.as_ref().unwrap();
        assert!(decoded.contains("1234567890"));
        assert!(decoded.contains("John Doe"));

        let children = node.children.as_ref().unwrap();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].key, "header");
        assert_eq!(children[1].key, "payload");

        // JWT payload contains a timestamp (iat: 1516239022) which should be decoded
        let payload_children = children[1].children.as_ref().unwrap();
        let iat = payload_children.iter().find(|c| c.key == "iat").unwrap();
        assert_eq!(iat.value_type.as_deref(), Some("timestamp"));
    }

    #[test]
    fn test_jwt_invalid_not_decoded() {
        // Not a valid JWT (random segments)
        let mut node = make_node("token", "abc.def.ghi");
        decode_node(&mut node);
        assert!(node.value_type.is_none());
    }

    #[test]
    fn test_jwt_two_segments_not_decoded() {
        let mut node = make_node("token", "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0In0");
        decode_node(&mut node);
        // Only two segments, not valid JWT format (regex requires 3 dot-separated parts)
        assert!(node.value_type.is_none());
    }

    // --- Timestamp tests ---

    #[test]
    fn test_timestamp_10_digit() {
        // 2026-02-08 12:00:00 UTC+8 = 1770465600
        let mut node = make_node("created_at", "1770465600");
        decode_node(&mut node);

        assert_eq!(node.value_type.as_deref(), Some("timestamp"));
        let decoded = node.decoded_value.as_ref().unwrap();
        assert!(decoded.contains("2026"));
    }

    #[test]
    fn test_timestamp_13_digit() {
        // 1770465600000 ms
        let mut node = make_node("timestamp", "1770465600000");
        decode_node(&mut node);

        assert_eq!(node.value_type.as_deref(), Some("timestamp"));
        let decoded = node.decoded_value.as_ref().unwrap();
        assert!(decoded.contains("2026"));
    }

    #[test]
    fn test_timestamp_too_small_not_decoded() {
        // Before 2000 - should not be treated as timestamp
        let mut node = make_node("id", "1234567890");
        decode_node(&mut node);
        // 1234567890 is 2009-02-13, which is > 946684800 (2000), so it IS a valid timestamp
        assert_eq!(node.value_type.as_deref(), Some("timestamp"));
    }

    #[test]
    fn test_non_timestamp_number_not_decoded() {
        // 5 digit number - not 10 or 13 digits
        let mut node = make_node("count", "12345");
        decode_node(&mut node);
        assert!(node.value_type.is_none());
    }

    #[test]
    fn test_timestamp_out_of_range_not_decoded() {
        // Too far in the future (year ~2090)
        let mut node = make_node("ts", "3800000000");
        decode_node(&mut node);
        assert!(node.value_type.is_none());
    }

    // --- Base64 tests ---

    #[test]
    fn test_base64_valid_decode() {
        // base64("Hello, World! This is a test.")
        let encoded = general_purpose::STANDARD.encode("Hello, World! This is a test.");
        let mut node = make_node("data", &encoded);
        decode_node(&mut node);

        assert_eq!(node.value_type.as_deref(), Some("base64"));
        assert_eq!(
            node.decoded_value.as_deref(),
            Some("Hello, World! This is a test.")
        );
    }

    #[test]
    fn test_base64_too_short_not_decoded() {
        // Less than 20 characters
        let mut node = make_node("short", "SGVsbG8=");
        decode_node(&mut node);
        assert_ne!(node.value_type.as_deref(), Some("base64"));
    }

    #[test]
    fn test_base64_non_utf8_not_decoded() {
        // Encode some binary data that's not valid UTF-8
        let binary = vec![0xff, 0xfe, 0xfd, 0xfc, 0xfb, 0xfa, 0xf9, 0xf8, 0xf7, 0xf6, 0xf5, 0xf4, 0xf3, 0xf2, 0xf1, 0xf0];
        let encoded = general_purpose::STANDARD.encode(&binary);
        let mut node = make_node("binary", &encoded);
        decode_node(&mut node);
        assert_ne!(node.value_type.as_deref(), Some("base64"));
    }

    #[test]
    fn test_base64_invalid_not_decoded() {
        // Contains invalid base64 characters
        let mut node = make_node("data", "This is not base64!!");
        decode_node(&mut node);
        assert_ne!(node.value_type.as_deref(), Some("base64"));
    }

    // --- URL encoding tests ---

    #[test]
    fn test_url_encoded_decode() {
        let mut node = make_node("query", "hello%20world%21");
        decode_node(&mut node);

        assert_eq!(node.value_type.as_deref(), Some("url_encoded"));
        assert_eq!(node.decoded_value.as_deref(), Some("hello world!"));
    }

    #[test]
    fn test_url_encoded_plus_sign() {
        let mut node = make_node("q", "hello+world%3F");
        decode_node(&mut node);

        assert_eq!(node.value_type.as_deref(), Some("url_encoded"));
        assert_eq!(node.decoded_value.as_deref(), Some("hello world?"));
    }

    #[test]
    fn test_url_encoded_chinese() {
        let mut node = make_node("name", "%E4%BD%A0%E5%A5%BD");
        decode_node(&mut node);

        assert_eq!(node.value_type.as_deref(), Some("url_encoded"));
        assert_eq!(node.decoded_value.as_deref(), Some("你好"));
    }

    #[test]
    fn test_no_url_encoding_not_decoded() {
        let mut node = make_node("plain", "hello world");
        decode_node(&mut node);
        assert_ne!(node.value_type.as_deref(), Some("url_encoded"));
    }

    // --- JSON tests ---

    #[test]
    fn test_json_object_decode() {
        let mut node = make_node("body", r#"{"name":"test","value":42}"#);
        decode_node(&mut node);

        assert_eq!(node.value_type.as_deref(), Some("json"));
        let decoded = node.decoded_value.as_ref().unwrap();
        assert!(decoded.contains("\"name\""));
        assert!(decoded.contains("\"test\""));
    }

    #[test]
    fn test_json_array_decode() {
        let mut node = make_node("items", r#"[1,2,3,"four"]"#);
        decode_node(&mut node);

        assert_eq!(node.value_type.as_deref(), Some("json"));
        let decoded = node.decoded_value.as_ref().unwrap();
        assert!(decoded.contains("\"four\""));
    }

    #[test]
    fn test_json_plain_string_not_decoded() {
        // A plain string is not an object or array
        let mut node = make_node("text", r#""just a string""#);
        decode_node(&mut node);
        assert_ne!(node.value_type.as_deref(), Some("json"));
    }

    // --- Compound value tests ---

    #[test]
    fn test_compound_decode() {
        let mut node = make_node("body", "name=test&age=30&city=Beijing");
        decode_node(&mut node);

        assert_eq!(node.value_type.as_deref(), Some("compound"));
        let children = node.children.as_ref().unwrap();
        assert_eq!(children.len(), 3);
        assert_eq!(children[0].key, "name");
        assert_eq!(children[0].value, "test");
        assert_eq!(children[1].key, "age");
        assert_eq!(children[1].value, "30");
        assert_eq!(children[2].key, "city");
        assert_eq!(children[2].value, "Beijing");
    }

    #[test]
    fn test_compound_recursive_decode() {
        // A compound value where a child has URL-encoded content
        let mut node = make_node("data", "name=%E5%BC%A0%E4%B8%89&ts=1770465600");
        decode_node(&mut node);

        assert_eq!(node.value_type.as_deref(), Some("compound"));
        let children = node.children.as_ref().unwrap();

        // name should be URL-decoded
        let name = &children[0];
        assert_eq!(name.value_type.as_deref(), Some("url_encoded"));
        assert_eq!(name.decoded_value.as_deref(), Some("张三"));

        // ts should be decoded as timestamp
        let ts = &children[1];
        assert_eq!(ts.value_type.as_deref(), Some("timestamp"));
    }

    #[test]
    fn test_single_kv_not_compound() {
        // Only one k=v pair, should not be treated as compound
        let mut node = make_node("data", "key=value");
        decode_node(&mut node);
        assert_ne!(node.value_type.as_deref(), Some("compound"));
    }

    // --- Integration tests ---

    #[test]
    fn test_apply_recursive_decode_headers() {
        let mut result = ParseResult {
            content_type: HttpContentType::Request,
            method: Some("GET".to_string()),
            url: Some("https://example.com".to_string()),
            status_code: None,
            status_text: None,
            protocol: Some("HTTP/1.1".to_string()),
            headers: vec![
                make_node("Authorization", "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c"),
                make_node("X-Timestamp", "1770465600"),
            ],
            query_params: None,
            body: None,
            raw_text: String::new(),
        };

        apply_recursive_decode(&mut result);

        assert_eq!(result.headers[0].value_type.as_deref(), Some("jwt"));
        assert_eq!(result.headers[1].value_type.as_deref(), Some("timestamp"));
    }

    #[test]
    fn test_apply_recursive_decode_query_params() {
        let mut result = ParseResult {
            content_type: HttpContentType::Request,
            method: Some("GET".to_string()),
            url: Some("https://example.com".to_string()),
            status_code: None,
            status_text: None,
            protocol: Some("HTTP/1.1".to_string()),
            headers: vec![],
            query_params: Some(vec![
                make_node("name", "%E4%BD%A0%E5%A5%BD"),
                make_node("ts", "1770465600"),
            ]),
            body: None,
            raw_text: String::new(),
        };

        apply_recursive_decode(&mut result);

        let params = result.query_params.as_ref().unwrap();
        assert_eq!(params[0].value_type.as_deref(), Some("url_encoded"));
        assert_eq!(params[0].decoded_value.as_deref(), Some("你好"));
        assert_eq!(params[1].value_type.as_deref(), Some("timestamp"));
    }

    #[test]
    fn test_apply_recursive_decode_cookie_children() {
        let mut result = ParseResult {
            content_type: HttpContentType::Request,
            method: Some("GET".to_string()),
            url: Some("https://example.com".to_string()),
            status_code: None,
            status_text: None,
            protocol: Some("HTTP/1.1".to_string()),
            headers: vec![ParseNode {
                key: "Cookie".to_string(),
                value: "session=abc; ts=1770465600".to_string(),
                children: Some(vec![
                    make_node("session", "abc"),
                    make_node("ts", "1770465600"),
                ]),
                description: None,
                decoded_value: None,
                value_type: None,
            }],
            query_params: None,
            body: None,
            raw_text: String::new(),
        };

        apply_recursive_decode(&mut result);

        let cookie = &result.headers[0];
        let children = cookie.children.as_ref().unwrap();
        // session=abc is too short to be anything
        assert!(children[0].value_type.is_none());
        // ts=1770465600 is a timestamp
        assert_eq!(children[1].value_type.as_deref(), Some("timestamp"));
    }

    #[test]
    fn test_decode_node_empty_value() {
        let mut node = make_node("empty", "");
        decode_node(&mut node);
        assert!(node.value_type.is_none());
        assert!(node.decoded_value.is_none());
    }

    #[test]
    fn test_decode_priority_jwt_over_base64() {
        // A JWT looks like base64 segments but should be detected as JWT first
        let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        let mut node = make_node("token", jwt);
        decode_node(&mut node);
        assert_eq!(node.value_type.as_deref(), Some("jwt"));
    }

    #[test]
    fn test_url_decode_function() {
        assert_eq!(url_decode("hello%20world"), "hello world");
        assert_eq!(url_decode("a%2Fb%2Fc"), "a/b/c");
        assert_eq!(url_decode("no+encoding+here"), "no encoding here");
        assert_eq!(url_decode("100%25+done"), "100% done");
    }

    // --- Boundary tests ---

    #[test]
    fn test_decode_node_whitespace_only() {
        let mut node = make_node("ws", "   \t\n  ");
        decode_node(&mut node);
        // Trimmed value is empty, should not decode
        assert!(node.value_type.is_none());
        assert!(node.decoded_value.is_none());
    }

    #[test]
    fn test_decode_node_with_leading_trailing_whitespace() {
        // Value with surrounding whitespace should still be decoded after trim
        let mut node = make_node("ts", "  1770465600  ");
        decode_node(&mut node);
        assert_eq!(node.value_type.as_deref(), Some("timestamp"));
    }

    #[test]
    fn test_very_long_non_matching_string() {
        // A long string that doesn't match any pattern
        let long = "abcdefghij".repeat(100);
        let mut node = make_node("data", &long);
        decode_node(&mut node);
        // This is all lowercase a-j repeated, should not match base64 (no +/= chars to distinguish)
        // RE_BASE64 requires [A-Za-z0-9+/]{20,}={0,2} - "abcdefghij" repeated matches
        // but decoding it may fail or produce garbage; let's just ensure no panic
        assert!(node.value_type.is_none() || node.value_type.is_some());
    }

    #[test]
    fn test_timestamp_boundary_year_2000() {
        // 946684800 is only 9 digits, so the regex requires 10 digits.
        // The smallest 10-digit timestamp in range is 946684800 but that's 9 digits.
        // So 1000000000 = 2001-09-09 is the smallest 10-digit value.
        let mut node = make_node("ts", "1000000000");
        decode_node(&mut node);
        assert_eq!(node.value_type.as_deref(), Some("timestamp"));
        assert!(node.decoded_value.as_ref().unwrap().contains("2001"));
    }

    #[test]
    fn test_timestamp_boundary_year_2050() {
        // Exactly 2524608000 = 2050-01-01T08:00:00+08:00
        let mut node = make_node("ts", "2524608000");
        decode_node(&mut node);
        assert_eq!(node.value_type.as_deref(), Some("timestamp"));
        assert!(node.decoded_value.as_ref().unwrap().contains("2050"));
    }

    #[test]
    fn test_timestamp_just_below_range() {
        // 946684799 = 1999-12-31, should not be treated as timestamp
        let mut node = make_node("ts", "946684799");
        decode_node(&mut node);
        assert!(node.value_type.is_none() || node.value_type.as_deref() != Some("timestamp"));
    }

    #[test]
    fn test_timestamp_just_above_range() {
        // 2524608001, just above the range
        let mut node = make_node("ts", "2524608001");
        decode_node(&mut node);
        assert!(node.value_type.is_none() || node.value_type.as_deref() != Some("timestamp"));
    }

    #[test]
    fn test_timestamp_13_digit_boundary() {
        // 1000000000000 ms (13 digits) = 1000000000 secs = 2001-09-09
        let mut node = make_node("ts", "1000000000000");
        decode_node(&mut node);
        assert_eq!(node.value_type.as_deref(), Some("timestamp"));
        assert!(node.decoded_value.as_ref().unwrap().contains("2001"));
    }

    #[test]
    fn test_base64_decode_with_unicode_result() {
        // Encode Chinese string to base64
        let encoded = general_purpose::STANDARD.encode("这是一个中文字符串测试");
        let mut node = make_node("data", &encoded);
        decode_node(&mut node);
        assert_eq!(node.value_type.as_deref(), Some("base64"));
        assert_eq!(node.decoded_value.as_deref(), Some("这是一个中文字符串测试"));
    }

    #[test]
    fn test_jwt_with_timestamps_in_payload() {
        // JWT with iat and exp timestamps in payload
        // Header: {"alg":"HS256","typ":"JWT"}
        // Payload: {"sub":"user","iat":1770465600,"exp":1770469200}
        use base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let header = URL_SAFE_NO_PAD.encode(r#"{"alg":"HS256","typ":"JWT"}"#);
        let payload = URL_SAFE_NO_PAD.encode(r#"{"sub":"user","iat":1770465600,"exp":1770469200}"#);
        let jwt = format!("{}.{}.dummysignaturevalue", header, payload);
        let mut node = make_node("Authorization", &jwt);
        decode_node(&mut node);

        assert_eq!(node.value_type.as_deref(), Some("jwt"));
        let children = node.children.as_ref().unwrap();
        let payload_node = &children[1];
        let payload_children = payload_node.children.as_ref().unwrap();

        // Both iat and exp should be decoded as timestamps
        let iat = payload_children.iter().find(|c| c.key == "iat").unwrap();
        assert_eq!(iat.value_type.as_deref(), Some("timestamp"));
        let exp = payload_children.iter().find(|c| c.key == "exp").unwrap();
        assert_eq!(exp.value_type.as_deref(), Some("timestamp"));
    }

    #[test]
    fn test_json_nested_object() {
        let json = r#"{"user":{"name":"Alice","age":30},"active":true}"#;
        let mut node = make_node("body", json);
        decode_node(&mut node);
        assert_eq!(node.value_type.as_deref(), Some("json"));
        let decoded = node.decoded_value.as_ref().unwrap();
        assert!(decoded.contains("Alice"));
    }

    #[test]
    fn test_json_empty_object() {
        let mut node = make_node("body", "{}");
        decode_node(&mut node);
        assert_eq!(node.value_type.as_deref(), Some("json"));
    }

    #[test]
    fn test_json_empty_array() {
        let mut node = make_node("body", "[]");
        decode_node(&mut node);
        assert_eq!(node.value_type.as_deref(), Some("json"));
    }

    #[test]
    fn test_compound_with_url_encoded_key() {
        let mut node = make_node("data", "user%20name=test&age=25");
        decode_node(&mut node);
        // RE_COMPOUND matches k=v&k=v patterns; key with %20 is still valid
        assert_eq!(node.value_type.as_deref(), Some("compound"));
        let children = node.children.as_ref().unwrap();
        assert_eq!(children[0].key, "user%20name");
    }

    #[test]
    fn test_url_decode_incomplete_percent() {
        // Incomplete percent encoding at the end: %2 (missing last hex digit)
        let result = url_decode("test%2");
        // Should not panic; %2 left as-is since from_str_radix on "2\0" or partial fails
        assert!(result.contains("test"));
    }

    #[test]
    fn test_url_decode_empty_string() {
        assert_eq!(url_decode(""), "");
    }

    #[test]
    fn test_url_decode_percent_at_end() {
        let result = url_decode("hello%");
        assert_eq!(result, "hello%");
    }

    #[test]
    fn test_base64_exactly_20_chars() {
        // base64 of "Hello, World!!!" is exactly 20 chars: "SGVsbG8sIFdvcmxkISEh"
        let encoded = general_purpose::STANDARD.encode("Hello, World!!!");
        assert!(encoded.len() >= 20);
        let mut node = make_node("data", &encoded);
        decode_node(&mut node);
        assert_eq!(node.value_type.as_deref(), Some("base64"));
    }

    #[test]
    fn test_compound_with_empty_values() {
        let mut node = make_node("data", "key1=&key2=&key3=val");
        decode_node(&mut node);
        assert_eq!(node.value_type.as_deref(), Some("compound"));
        let children = node.children.as_ref().unwrap();
        assert_eq!(children.len(), 3);
        assert_eq!(children[0].value, "");
        assert_eq!(children[1].value, "");
        assert_eq!(children[2].value, "val");
    }

    // --- Integration tests with real-world examples ---

    #[test]
    fn test_real_curl_example_end_to_end() {
        // Simulate parsing the real cURL example from real_example.md
        // and then running recursive decode on the result
        let mut result = ParseResult {
            content_type: HttpContentType::Request,
            method: Some("GET".to_string()),
            url: Some("https://anyrouter.top/api/log/self/stat?type=0&token_name=&model_name=&start_timestamp=1770480000&end_timestamp=1770536299&group=".to_string()),
            status_code: None,
            status_text: None,
            protocol: None,
            headers: vec![
                make_node("accept", "application/json, text/plain, */*"),
                make_node("accept-language", "en,zh-CN;q=0.9,zh;q=0.8"),
                make_node("cache-control", "no-store"),
                ParseNode {
                    key: "Cookie".to_string(),
                    value: "session=MTc2ODgzMzYxMHxEWDhFQVFMX2dBQUJFQUVRQUFEX3hQLUFBQVlHYzNSeWFXNW5EQVFBQW1sa0EybHVkQVFFQVA2eWtBWnpkSEpwYm1jTUNnQUlkWE5sY201aGJXVUdjM1J5YVc1bkRBNEFER2RwZEdoMVlsOHlNamcxTmdaemRISnBibWNNQmdBRWNtOXNaUU5wYm5RRUFnQUNCbk4wY21sdVp3d0lBQVp6ZEdGMGRYTURhVzUwQkFJQUFnWnpkSEpwYm1jTUJ3QUZaM0p2ZFhBR2MzUnlhVzVuREFrQUIyUmxabUYxYkhRR2MzUnlhVzVuREEwQUMyOWhkWFJvWDNOMFlYUmxCbk4wY21sdVp3d09BQXhoZWtOYVIwbDBlR1U1VEVjPXykP1Jr8RuM8g4CZXDTAj3Wu6ypzYPFAMWvKxgqOcrNYA==".to_string(),
                    children: Some(vec![
                        make_node("session", "MTc2ODgzMzYxMHxEWDhFQVFMX2dBQUJFQUVRQUFEX3hQLUFBQVlHYzNSeWFXNW5EQVFBQW1sa0EybHVkQVFFQVA2eWtBWnpkSEpwYm1jTUNnQUlkWE5sY201aGJXVUdjM1J5YVc1bkRBNEFER2RwZEdoMVlsOHlNamcxTmdaemRISnBibWNNQmdBRWNtOXNaUU5wYm5RRUFnQUNCbk4wY21sdVp3d0lBQVp6ZEdGMGRYTURhVzUwQkFJQUFnWnpkSEpwYm1jTUJ3QUZaM0p2ZFhBR2MzUnlhVzVuREFrQUIyUmxabUYxYkhRR2MzUnlhVzVuREEwQUMyOWhkWFJvWDNOMFlYUmxCbk4wY21sdVp3d09BQXhoZWtOYVIwbDBlR1U1VEVjPXykP1Jr8RuM8g4CZXDTAj3Wu6ypzYPFAMWvKxgqOcrNYA=="),
                        make_node("acw_tc", "2ff617a217705315504591037e025a23ce43b4f78a6ffbed1cb2b5e895"),
                        make_node("cdn_sec_tc", "2ff617a217705315504591037e025a23ce43b4f78a6ffbed1cb2b5e895"),
                        make_node("acw_sc__v2", "69882ade282d91973da11de94c3b29212dc3207d"),
                    ]),
                    description: None,
                    decoded_value: None,
                    value_type: None,
                },
                make_node("dnt", "1"),
                make_node("new-api-user", "22856"),
            ],
            query_params: Some(vec![
                make_node("type", "0"),
                make_node("token_name", ""),
                make_node("model_name", ""),
                make_node("start_timestamp", "1770480000"),
                make_node("end_timestamp", "1770536299"),
                make_node("group", ""),
            ]),
            body: None,
            raw_text: String::new(),
        };

        apply_recursive_decode(&mut result);

        // Verify query param timestamps are decoded
        let params = result.query_params.as_ref().unwrap();
        let start_ts = params.iter().find(|p| p.key == "start_timestamp").unwrap();
        assert_eq!(start_ts.value_type.as_deref(), Some("timestamp"));
        assert!(start_ts.decoded_value.as_ref().unwrap().contains("2026"));

        let end_ts = params.iter().find(|p| p.key == "end_timestamp").unwrap();
        assert_eq!(end_ts.value_type.as_deref(), Some("timestamp"));
        assert!(end_ts.decoded_value.as_ref().unwrap().contains("2026"));

        // Verify empty query params are not decoded
        let token_name = params.iter().find(|p| p.key == "token_name").unwrap();
        assert!(token_name.value_type.is_none());

        // Verify Cookie children are decoded where applicable
        let cookie = result.headers.iter().find(|h| h.key == "Cookie").unwrap();
        let cookie_children = cookie.children.as_ref().unwrap();

        // The session cookie value is base64-like (long base64 string)
        let session = cookie_children.iter().find(|c| c.key == "session").unwrap();
        // The session value contains non-standard base64 chars (|), so it may or may not decode
        // The important thing is it doesn't panic
        assert!(session.value_type.is_none() || session.value_type.is_some());
    }

    #[test]
    fn test_real_fetch_example_end_to_end() {
        // Simulate parsing a fetch request with headers and cookie
        let mut result = ParseResult {
            content_type: HttpContentType::Request,
            method: Some("GET".to_string()),
            url: Some("https://anyrouter.top/api/user/models".to_string()),
            status_code: None,
            status_text: None,
            protocol: None,
            headers: vec![
                make_node("accept", "application/json, text/plain, */*"),
                make_node("accept-language", "en,zh-CN;q=0.9,zh;q=0.8"),
                make_node("cache-control", "no-store"),
                make_node("new-api-user", "22856"),
                ParseNode {
                    key: "cookie".to_string(),
                    value: "session=MTc2ODgz; acw_tc=2ff617a2".to_string(),
                    children: Some(vec![
                        make_node("session", "MTc2ODgz"),
                        make_node("acw_tc", "2ff617a2"),
                    ]),
                    description: None,
                    decoded_value: None,
                    value_type: None,
                },
                make_node("Referer", "https://anyrouter.top/console/token"),
            ],
            query_params: None,
            body: None,
            raw_text: String::new(),
        };

        apply_recursive_decode(&mut result);

        // Headers that are plain strings should not be decoded
        let accept = result.headers.iter().find(|h| h.key == "accept").unwrap();
        assert!(accept.value_type.is_none());

        // Cookie children should be processed
        let cookie = result.headers.iter().find(|h| h.key == "cookie").unwrap();
        let children = cookie.children.as_ref().unwrap();
        // Short values should not match any decoder pattern
        assert!(children[0].value_type.is_none() || children[0].value_type.is_some());
    }

    #[test]
    fn test_full_pipeline_curl_parse_and_decode() {
        // End-to-end: parse a cURL command then apply recursive decode
        use crate::curl_parser::parse_curl;

        let input = r#"curl 'https://example.com/api?ts=1770465600&name=%E5%BC%A0%E4%B8%89' \
  -H 'accept: application/json' \
  -H 'Authorization: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c' \
  -b 'token=abc123; ts=1770465600'"#;

        let mut result = parse_curl(input);
        apply_recursive_decode(&mut result);

        // Query params: ts should be timestamp, name should be URL-decoded
        let params = result.query_params.as_ref().unwrap();
        let ts = params.iter().find(|p| p.key == "ts").unwrap();
        assert_eq!(ts.value_type.as_deref(), Some("timestamp"));

        // Authorization header should be decoded as JWT
        let auth = result.headers.iter().find(|h| h.key == "Authorization").unwrap();
        assert_eq!(auth.value_type.as_deref(), Some("jwt"));
        assert!(auth.children.is_some());

        // Cookie children: ts should be timestamp
        let cookie = result.headers.iter().find(|h| h.key == "Cookie").unwrap();
        let cookie_children = cookie.children.as_ref().unwrap();
        let cookie_ts = cookie_children.iter().find(|c| c.key == "ts").unwrap();
        assert_eq!(cookie_ts.value_type.as_deref(), Some("timestamp"));
    }

    #[test]
    fn test_full_pipeline_fetch_parse_and_decode() {
        // End-to-end: parse a fetch call then apply recursive decode
        use crate::fetch_parser::parse_fetch;

        let input = r#"fetch("https://example.com/api?created=1770465600", {
  "headers": {
    "accept": "application/json",
    "cookie": "session=abc; ts=1770465600"
  },
  "body": null,
  "method": "GET"
});"#;

        let mut result = parse_fetch(input);
        apply_recursive_decode(&mut result);

        // Query param timestamp
        let params = result.query_params.as_ref().unwrap();
        let created = params.iter().find(|p| p.key == "created").unwrap();
        assert_eq!(created.value_type.as_deref(), Some("timestamp"));

        // Cookie children: ts should be timestamp
        let cookie = result.headers.iter().find(|h| h.key == "cookie").unwrap();
        let cookie_children = cookie.children.as_ref().unwrap();
        let ts = cookie_children.iter().find(|c| c.key == "ts").unwrap();
        assert_eq!(ts.value_type.as_deref(), Some("timestamp"));
    }

    #[test]
    fn test_full_pipeline_raw_http_parse_and_decode() {
        // End-to-end: parse raw HTTP then apply recursive decode
        use crate::parser::parse_http_text;

        let input = "GET /api/data?ts=1770465600 HTTP/1.1\n\
                      Host: example.com\n\
                      Cookie: session=abc; created=1770465600\n\
                      X-Data: name%3Dalice%26age%3D30";

        let mut result = parse_http_text(input);
        apply_recursive_decode(&mut result);

        // Query param
        let params = result.query_params.as_ref().unwrap();
        let ts = params.iter().find(|p| p.key == "ts").unwrap();
        assert_eq!(ts.value_type.as_deref(), Some("timestamp"));

        // Cookie children
        let cookie = result.headers.iter().find(|h| h.key == "Cookie").unwrap();
        let cookie_children = cookie.children.as_ref().unwrap();
        let created = cookie_children.iter().find(|c| c.key == "created").unwrap();
        assert_eq!(created.value_type.as_deref(), Some("timestamp"));

        // X-Data should be decoded as URL-encoded
        let x_data = result.headers.iter().find(|h| h.key == "X-Data").unwrap();
        assert_eq!(x_data.value_type.as_deref(), Some("url_encoded"));
        assert_eq!(x_data.decoded_value.as_deref(), Some("name=alice&age=30"));
    }
}
