use regex::Regex;
use std::sync::LazyLock;
use url::Url;

use crate::models::{HttpContentType, ParseNode, ParseResult};

static RE_REQUEST_LINE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(GET|POST|PUT|DELETE|PATCH|HEAD|OPTIONS|TRACE|CONNECT)\s+(\S+)(?:\s+HTTP/(\d\.\d))?\s*$",
    )
    .unwrap()
});

static RE_RESPONSE_LINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^HTTP/(\d\.\d)\s+(\d{3})\s*(.*)$").unwrap());

static RE_HEADER_LINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([\w-]+):\s*(.*)$").unwrap());

/// 将原始 HTTP 文本解析为结构化的 ParseResult。
pub fn parse_http_text(raw: &str) -> ParseResult {
    let raw_text = raw.to_string();
    let trimmed = raw.trim();
    let mut lines = trimmed.lines();

    let first_line = lines.next().unwrap_or("");

    let content_type;
    let mut method = None;
    let mut url_str = None;
    let mut status_code = None;
    let mut status_text = None;
    let mut protocol = None;
    let header_start_lines: Vec<&str>;

    if let Some(caps) = RE_REQUEST_LINE.captures(first_line) {
        content_type = HttpContentType::Request;
        method = Some(caps[1].to_string());
        url_str = Some(caps[2].to_string());
        protocol = caps.get(3).map(|m| format!("HTTP/{}", m.as_str()));
        header_start_lines = lines.collect();
    } else if let Some(caps) = RE_RESPONSE_LINE.captures(first_line) {
        content_type = HttpContentType::Response;
        protocol = Some(format!("HTTP/{}", &caps[1]));
        status_code = caps[2].parse::<u16>().ok();
        status_text = Some(caps[3].trim().to_string());
        header_start_lines = lines.collect();
    } else {
        content_type = HttpContentType::HeadersOnly;
        header_start_lines = trimmed.lines().collect();
    }

    // 空行分割 headers 和 body
    let mut headers_raw: Vec<&str> = Vec::new();
    let mut body_lines: Vec<&str> = Vec::new();
    let mut found_blank = false;

    for line in &header_start_lines {
        if !found_blank {
            if line.trim().is_empty() {
                found_blank = true;
            } else {
                headers_raw.push(line);
            }
        } else {
            body_lines.push(line);
        }
    }

    let headers: Vec<ParseNode> = headers_raw
        .iter()
        .filter_map(|line| {
            RE_HEADER_LINE.captures(line).map(|caps| {
                let key = caps[1].to_string();
                let value = caps[2].to_string();
                let children = parse_header_value_children(&key, &value);
                ParseNode {
                    key,
                    value,
                    children,
                    description: None,
                }
            })
        })
        .collect();

    let query_params = url_str.as_ref().and_then(|u| parse_query_params(u));

    let body = if body_lines.is_empty() {
        None
    } else {
        Some(body_lines.join("\n"))
    };

    ParseResult {
        content_type,
        method,
        url: url_str,
        status_code,
        status_text,
        protocol,
        headers,
        query_params,
        body,
        raw_text,
    }
}

/// 从 URL 或路径字符串中解析查询参数。
fn parse_query_params(url_or_path: &str) -> Option<Vec<ParseNode>> {
    let parsed = Url::parse(url_or_path)
        .or_else(|_| Url::parse(&format!("http://dummy{}", url_or_path)));

    match parsed {
        Ok(url) => {
            let params: Vec<ParseNode> = url
                .query_pairs()
                .map(|(k, v)| ParseNode {
                    key: k.into_owned(),
                    value: v.into_owned(),
                    children: None,
                    description: None,
                })
                .collect();
            if params.is_empty() {
                None
            } else {
                Some(params)
            }
        }
        Err(_) => None,
    }
}

/// Phase 1 的 Header Value 子解析。
/// 处理 Cookie 风格的 "k1=v1; k2=v2" 模式。
fn parse_header_value_children(key: &str, value: &str) -> Option<Vec<ParseNode>> {
    let lower_key = key.to_lowercase();

    if lower_key == "cookie" || lower_key == "set-cookie" {
        let children: Vec<ParseNode> = value
            .split(';')
            .map(|pair| pair.trim())
            .filter(|pair| !pair.is_empty())
            .map(|pair| {
                if let Some((k, v)) = pair.split_once('=') {
                    ParseNode {
                        key: k.trim().to_string(),
                        value: v.trim().to_string(),
                        children: None,
                        description: None,
                    }
                } else {
                    // Cookie 标志如 HttpOnly, Secure
                    ParseNode {
                        key: pair.to_string(),
                        value: String::new(),
                        children: None,
                        description: None,
                    }
                }
            })
            .collect();
        if children.is_empty() {
            None
        } else {
            Some(children)
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_get_request() {
        let input = "GET /api/users?page=1&limit=10 HTTP/1.1\nHost: example.com\nAccept: application/json";
        let result = parse_http_text(input);

        assert!(matches!(result.content_type, HttpContentType::Request));
        assert_eq!(result.method.as_deref(), Some("GET"));
        assert_eq!(result.url.as_deref(), Some("/api/users?page=1&limit=10"));
        assert_eq!(result.protocol.as_deref(), Some("HTTP/1.1"));
        assert_eq!(result.headers.len(), 2);
        assert_eq!(result.headers[0].key, "Host");
        assert_eq!(result.headers[0].value, "example.com");
    }

    #[test]
    fn test_parse_post_with_body() {
        let input = "POST /api/data HTTP/1.1\nContent-Type: application/json\n\n{\"name\": \"test\"}";
        let result = parse_http_text(input);

        assert!(matches!(result.content_type, HttpContentType::Request));
        assert_eq!(result.method.as_deref(), Some("POST"));
        assert_eq!(result.headers.len(), 1);
        assert_eq!(result.body.as_deref(), Some("{\"name\": \"test\"}"));
    }

    #[test]
    fn test_parse_response() {
        let input = "HTTP/1.1 200 OK\nContent-Type: text/html\nContent-Length: 1234";
        let result = parse_http_text(input);

        assert!(matches!(result.content_type, HttpContentType::Response));
        assert_eq!(result.status_code, Some(200));
        assert_eq!(result.status_text.as_deref(), Some("OK"));
        assert_eq!(result.protocol.as_deref(), Some("HTTP/1.1"));
        assert_eq!(result.headers.len(), 2);
    }

    #[test]
    fn test_parse_response_404() {
        let input = "HTTP/1.1 404 Not Found\nServer: nginx";
        let result = parse_http_text(input);

        assert!(matches!(result.content_type, HttpContentType::Response));
        assert_eq!(result.status_code, Some(404));
        assert_eq!(result.status_text.as_deref(), Some("Not Found"));
    }

    #[test]
    fn test_parse_query_params() {
        let input = "GET /search?q=hello%20world&lang=zh HTTP/1.1\nHost: google.com";
        let result = parse_http_text(input);

        let params = result.query_params.unwrap();
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].key, "q");
        assert_eq!(params[0].value, "hello world");
        assert_eq!(params[1].key, "lang");
        assert_eq!(params[1].value, "zh");
    }

    #[test]
    fn test_parse_cookie_children() {
        let input = "Host: example.com\nCookie: session_id=abc123; theme=dark; lang=zh-CN";
        let result = parse_http_text(input);

        let cookie_header = result.headers.iter().find(|h| h.key == "Cookie").unwrap();
        let children = cookie_header.children.as_ref().unwrap();
        assert_eq!(children.len(), 3);
        assert_eq!(children[0].key, "session_id");
        assert_eq!(children[0].value, "abc123");
        assert_eq!(children[1].key, "theme");
        assert_eq!(children[1].value, "dark");
    }

    #[test]
    fn test_parse_set_cookie_with_flags() {
        let input =
            "Host: example.com\nSet-Cookie: token=xyz; Path=/; HttpOnly; Secure; Max-Age=3600";
        let result = parse_http_text(input);

        let sc = result
            .headers
            .iter()
            .find(|h| h.key == "Set-Cookie")
            .unwrap();
        let children = sc.children.as_ref().unwrap();
        assert_eq!(children[0].key, "token");
        assert_eq!(children[0].value, "xyz");
        // HttpOnly 和 Secure 作为没有值的标志
        let http_only = children.iter().find(|c| c.key == "HttpOnly").unwrap();
        assert_eq!(http_only.value, "");
    }

    #[test]
    fn test_parse_headers_only() {
        let input = "Content-Type: application/json\nAuthorization: Bearer token123";
        let result = parse_http_text(input);

        assert!(matches!(result.content_type, HttpContentType::HeadersOnly));
        assert_eq!(result.headers.len(), 2);
        assert!(result.method.is_none());
        assert!(result.status_code.is_none());
    }

    #[test]
    fn test_parse_no_query_params() {
        let input = "GET /api/users HTTP/1.1\nHost: example.com";
        let result = parse_http_text(input);

        assert!(result.query_params.is_none());
    }

    #[test]
    fn test_raw_text_preserved() {
        let input = "GET / HTTP/1.1\nHost: localhost";
        let result = parse_http_text(input);
        assert_eq!(result.raw_text, input);
    }
}
