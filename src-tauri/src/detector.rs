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
    LazyLock::new(|| Regex::new(r"^[\w-]+:\s*.+$").unwrap());

/// 判断文本是否像 HTTP 请求/响应数据。
///
/// 三级检测策略：
/// 1. 首行匹配 HTTP 请求行 (如 "GET /api HTTP/1.1")
/// 2. 首行匹配 HTTP 响应行 (如 "HTTP/1.1 200 OK")
/// 3. 包含 ≥2 行 Header 格式 ("Key: Value")
pub fn is_http_like(text: &str) -> bool {
    let text = text.trim();
    if text.is_empty() || text.len() < 10 {
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

    #[test]
    fn test_get_request() {
        assert!(is_http_like("GET /api/users HTTP/1.1\nHost: example.com"));
    }

    #[test]
    fn test_post_request() {
        assert!(is_http_like(
            "POST /api/data HTTP/1.1\nContent-Type: application/json"
        ));
    }

    #[test]
    fn test_request_without_version() {
        assert!(is_http_like(
            "GET /api/users\nHost: example.com\nAccept: text/html"
        ));
    }

    #[test]
    fn test_response_line() {
        assert!(is_http_like(
            "HTTP/1.1 200 OK\nContent-Type: text/html\nContent-Length: 1234"
        ));
    }

    #[test]
    fn test_response_404() {
        assert!(is_http_like("HTTP/1.1 404 Not Found\nServer: nginx"));
    }

    #[test]
    fn test_headers_only() {
        assert!(is_http_like(
            "Content-Type: text/html\nCache-Control: no-cache"
        ));
    }

    #[test]
    fn test_single_header_rejected() {
        assert!(!is_http_like("Content-Type: text/html"));
    }

    #[test]
    fn test_plain_text_rejected() {
        assert!(!is_http_like("Hello world, this is just a test message"));
    }

    #[test]
    fn test_empty_input() {
        assert!(!is_http_like(""));
    }

    #[test]
    fn test_short_input() {
        assert!(!is_http_like("GET"));
    }

    #[test]
    fn test_whitespace_only() {
        assert!(!is_http_like("   \n  \n  "));
    }

    #[test]
    fn test_many_headers() {
        let input = "Host: example.com\nAccept: */*\nUser-Agent: curl/7.68\nAuthorization: Bearer xyz";
        assert!(is_http_like(input));
    }
}
