use regex::Regex;
use std::sync::LazyLock;

static RE_REQUEST_LINE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(GET|POST|PUT|DELETE|PATCH|HEAD|OPTIONS|TRACE|CONNECT)\s+\S+(\s+HTTP/\d\.\d)?\s*$",
    )
    .unwrap()
});

static RE_RESPONSE_LINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^HTTP/\d\.\d\s+\d{3}\s+.*$").unwrap());

static RE_HEADER_LINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[\w-]+:\s*.*$").unwrap());

/// 输入格式类型
#[derive(Debug, Clone, PartialEq)]
pub enum InputFormat {
    Curl,
    Fetch,
    RawHttp,
    Unknown,
}

/// 自动检测输入文本的格式。
pub fn detect_input_format(text: &str) -> InputFormat {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return InputFormat::Unknown;
    }

    if trimmed.starts_with("curl ") || trimmed.starts_with("curl\t") {
        return InputFormat::Curl;
    }

    if trimmed.starts_with("fetch(") || trimmed.starts_with("fetch (") {
        return InputFormat::Fetch;
    }

    if is_raw_http_like(trimmed) {
        return InputFormat::RawHttp;
    }

    InputFormat::Unknown
}

/// 判断文本是否像 HTTP 请求/响应数据（包括 cURL 和 fetch 格式）。
pub fn is_http_like(text: &str) -> bool {
    detect_input_format(text) != InputFormat::Unknown
}

/// 判断文本是否像原始 HTTP 格式。
fn is_raw_http_like(text: &str) -> bool {
    if text.len() < 10 {
        return false;
    }

    let first_line = text.lines().next().unwrap_or("");

    if RE_REQUEST_LINE.is_match(first_line) {
        return true;
    }

    if RE_RESPONSE_LINE.is_match(first_line) {
        return true;
    }

    let header_line_count = text
        .lines()
        .take(20)
        .filter(|line| RE_HEADER_LINE.is_match(line))
        .count();

    header_line_count >= 2
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- detect_input_format 测试 ---

    #[test]
    fn test_detect_curl() {
        assert_eq!(
            detect_input_format("curl 'https://example.com' -H 'accept: */*'"),
            InputFormat::Curl
        );
    }

    #[test]
    fn test_detect_curl_multiline() {
        assert_eq!(
            detect_input_format("curl 'https://example.com' \\\n  -H 'accept: */*'"),
            InputFormat::Curl
        );
    }

    #[test]
    fn test_detect_fetch() {
        assert_eq!(
            detect_input_format(r#"fetch("https://example.com", { "method": "GET" });"#),
            InputFormat::Fetch
        );
    }

    #[test]
    fn test_detect_fetch_with_space() {
        assert_eq!(
            detect_input_format(r#"fetch ("https://example.com")"#),
            InputFormat::Fetch
        );
    }

    #[test]
    fn test_detect_raw_http_request() {
        assert_eq!(
            detect_input_format("GET /api/users HTTP/1.1\nHost: example.com"),
            InputFormat::RawHttp
        );
    }

    #[test]
    fn test_detect_raw_http_response() {
        assert_eq!(
            detect_input_format("HTTP/1.1 200 OK\nContent-Type: text/html"),
            InputFormat::RawHttp
        );
    }

    #[test]
    fn test_detect_unknown() {
        assert_eq!(
            detect_input_format("Hello world, this is just text"),
            InputFormat::Unknown
        );
    }

    #[test]
    fn test_detect_empty() {
        assert_eq!(detect_input_format(""), InputFormat::Unknown);
    }

    // --- is_http_like 兼容测试 ---

    #[test]
    fn test_is_http_like_curl() {
        assert!(is_http_like("curl 'https://example.com' -H 'accept: */*'"));
    }

    #[test]
    fn test_is_http_like_fetch() {
        assert!(is_http_like(r#"fetch("https://example.com")"#));
    }

    #[test]
    fn test_is_http_like_raw() {
        assert!(is_http_like(
            "GET /api/users HTTP/1.1\nHost: example.com"
        ));
    }

    #[test]
    fn test_is_http_like_headers_only() {
        assert!(is_http_like(
            "Content-Type: text/html\nCache-Control: no-cache"
        ));
    }

    #[test]
    fn test_is_http_like_rejects_plain_text() {
        assert!(!is_http_like("Hello world, this is just a test message"));
    }

    #[test]
    fn test_is_http_like_rejects_empty() {
        assert!(!is_http_like(""));
    }

    #[test]
    fn test_is_http_like_rejects_short() {
        assert!(!is_http_like("GET"));
    }
}
