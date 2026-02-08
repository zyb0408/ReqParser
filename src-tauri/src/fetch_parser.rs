use url::Url;

use crate::models::{HttpContentType, ParseNode, ParseResult};

/// 解析浏览器 DevTools "Copy as fetch" 格式的文本。
pub fn parse_fetch(input: &str) -> ParseResult {
    let raw_text = input.to_string();
    let trimmed = input.trim();

    // 提取 fetch(...) 内部内容
    let inner = match extract_fetch_inner(trimmed) {
        Some(s) => s,
        None => {
            return empty_result(raw_text);
        }
    };

    // 分离 URL 和 options
    let (url_str, options_json) = split_url_and_options(inner);

    // 解析 options JSON
    let options: Option<serde_json::Value> = options_json
        .and_then(|json_str| serde_json::from_str(json_str).ok());

    // 提取 method
    let method = options
        .as_ref()
        .and_then(|o| o.get("method"))
        .and_then(|v| v.as_str())
        .unwrap_or("GET")
        .to_string();

    // 提取 headers
    let headers = options
        .as_ref()
        .and_then(|o| o.get("headers"))
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .map(|(key, val)| {
                    let value = val.as_str().unwrap_or("").to_string();
                    let children = parse_cookie_children(key, &value);
                    ParseNode {
                        key: key.clone(),
                        value,
                        children,
                        description: None,
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    // 提取 body
    let body = options
        .as_ref()
        .and_then(|o| o.get("body"))
        .and_then(|v| {
            if v.is_null() {
                None
            } else {
                // body 可能是字符串或其他 JSON 值
                v.as_str()
                    .map(|s| s.to_string())
                    .or_else(|| Some(v.to_string()))
            }
        });

    // 解析 URL query params
    let query_params = parse_query_params(&url_str);

    ParseResult {
        content_type: HttpContentType::Request,
        method: Some(method),
        url: Some(url_str),
        status_code: None,
        status_text: None,
        protocol: None,
        headers,
        query_params,
        body,
        raw_text,
    }
}

/// 提取 `fetch(` 和最后一个 `)` 之间的内容。
/// 支持末尾带或不带分号。
fn extract_fetch_inner(input: &str) -> Option<&str> {
    // 找到 fetch( 的位置
    let start = input.find("fetch(")? + "fetch(".len();
    // 找到最后一个 ) 的位置（去掉末尾分号后）
    let trimmed = input.trim_end_matches(';').trim_end();
    let end = trimmed.rfind(')')?;
    if end < start {
        return None;
    }
    Some(&trimmed[start..end])
}

/// 将 fetch 内部参数分离为 URL 字符串和可选的 options JSON。
/// 输入形如: `"url", {...}` 或 `"url"`
fn split_url_and_options(inner: &str) -> (String, Option<&str>) {
    let trimmed = inner.trim();

    // URL 是第一个引号包裹的字符串
    let url_str = extract_quoted_string(trimmed);

    // 找到 URL 字符串结束后的位置
    // 寻找第一个引号对之后的逗号
    let after_url = skip_quoted_string(trimmed);

    let options = after_url.and_then(|rest| {
        // 跳过逗号和空白
        let rest = rest.trim_start_matches(',').trim();
        if rest.is_empty() {
            None
        } else {
            Some(rest)
        }
    });

    (url_str, options)
}

/// 提取第一个引号包裹的字符串内容（处理转义引号）。
fn extract_quoted_string(input: &str) -> String {
    let trimmed = input.trim();
    if !trimmed.starts_with('"') && !trimmed.starts_with('\'') {
        // 没有引号，取到逗号或结尾
        return trimmed
            .split(',')
            .next()
            .unwrap_or(trimmed)
            .trim()
            .to_string();
    }

    let quote_char = trimmed.as_bytes()[0];
    let bytes = trimmed.as_bytes();
    let mut i = 1;
    let mut result = String::new();

    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() {
            // 转义字符
            result.push(bytes[i + 1] as char);
            i += 2;
        } else if bytes[i] == quote_char {
            break;
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }

    result
}

/// 跳过第一个引号包裹的字符串，返回剩余部分。
fn skip_quoted_string(input: &str) -> Option<&str> {
    let trimmed = input.trim();
    if !trimmed.starts_with('"') && !trimmed.starts_with('\'') {
        // 没有引号包裹，找逗号
        return trimmed.find(',').map(|pos| &trimmed[pos..]);
    }

    let quote_char = trimmed.as_bytes()[0];
    let bytes = trimmed.as_bytes();
    let mut i = 1;

    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() {
            i += 2;
        } else if bytes[i] == quote_char {
            return Some(&trimmed[i + 1..]);
        } else {
            i += 1;
        }
    }

    None
}

/// Cookie 子解析：按 `;` 拆解 cookie 值。
fn parse_cookie_children(key: &str, value: &str) -> Option<Vec<ParseNode>> {
    let lower_key = key.to_lowercase();
    if lower_key != "cookie" && lower_key != "set-cookie" {
        return None;
    }

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
}

/// 从 URL 解析 query params。
fn parse_query_params(url_str: &str) -> Option<Vec<ParseNode>> {
    let parsed = Url::parse(url_str)
        .or_else(|_| Url::parse(&format!("http://dummy{}", url_str)));

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

/// 返回一个空的 ParseResult（解析失败时使用）。
fn empty_result(raw_text: String) -> ParseResult {
    ParseResult {
        content_type: HttpContentType::Unknown,
        method: None,
        url: None,
        status_code: None,
        status_text: None,
        protocol: None,
        headers: vec![],
        query_params: None,
        body: None,
        raw_text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_get_with_headers() {
        let input = r#"fetch("https://example.com/api/data", {
  "headers": {
    "accept": "application/json",
    "user-agent": "Mozilla/5.0"
  },
  "body": null,
  "method": "GET"
});"#;

        let result = parse_fetch(input);

        assert!(matches!(result.content_type, HttpContentType::Request));
        assert_eq!(result.method.as_deref(), Some("GET"));
        assert_eq!(result.url.as_deref(), Some("https://example.com/api/data"));
        assert_eq!(result.headers.len(), 2);
        assert_eq!(result.headers[0].key, "accept");
        assert_eq!(result.headers[0].value, "application/json");
        assert!(result.body.is_none());
    }

    #[test]
    fn test_cookie_children() {
        let input = r#"fetch("https://example.com/api", {
  "headers": {
    "cookie": "session=abc123; theme=dark; lang=zh-CN"
  },
  "body": null,
  "method": "GET"
});"#;

        let result = parse_fetch(input);

        let cookie_header = result
            .headers
            .iter()
            .find(|h| h.key == "cookie")
            .unwrap();
        let children = cookie_header.children.as_ref().unwrap();
        assert_eq!(children.len(), 3);
        assert_eq!(children[0].key, "session");
        assert_eq!(children[0].value, "abc123");
        assert_eq!(children[1].key, "theme");
        assert_eq!(children[1].value, "dark");
        assert_eq!(children[2].key, "lang");
        assert_eq!(children[2].value, "zh-CN");
    }

    #[test]
    fn test_post_with_body() {
        let input = r#"fetch("https://example.com/api/submit", {
  "headers": {
    "content-type": "application/json"
  },
  "body": "{\"name\":\"test\",\"value\":42}",
  "method": "POST"
});"#;

        let result = parse_fetch(input);

        assert_eq!(result.method.as_deref(), Some("POST"));
        assert_eq!(
            result.body.as_deref(),
            Some("{\"name\":\"test\",\"value\":42}")
        );
    }

    #[test]
    fn test_url_query_params() {
        let input = r#"fetch("https://example.com/search?q=hello&page=1&lang=zh", {
  "headers": {},
  "body": null,
  "method": "GET"
});"#;

        let result = parse_fetch(input);

        let params = result.query_params.unwrap();
        assert_eq!(params.len(), 3);
        assert_eq!(params[0].key, "q");
        assert_eq!(params[0].value, "hello");
        assert_eq!(params[1].key, "page");
        assert_eq!(params[1].value, "1");
        assert_eq!(params[2].key, "lang");
        assert_eq!(params[2].value, "zh");
    }

    #[test]
    fn test_body_null() {
        let input = r#"fetch("https://example.com/api", {
  "headers": {},
  "body": null,
  "method": "GET"
});"#;

        let result = parse_fetch(input);
        assert!(result.body.is_none());
    }

    #[test]
    fn test_fetch_url_only() {
        let input = r#"fetch("https://example.com/api/simple")"#;

        let result = parse_fetch(input);

        assert!(matches!(result.content_type, HttpContentType::Request));
        assert_eq!(result.method.as_deref(), Some("GET"));
        assert_eq!(
            result.url.as_deref(),
            Some("https://example.com/api/simple")
        );
        assert!(result.headers.is_empty());
        assert!(result.body.is_none());
    }

    #[test]
    fn test_method_defaults_to_get() {
        let input = r#"fetch("https://example.com/api", {
  "headers": {
    "accept": "*/*"
  },
  "body": null
});"#;

        let result = parse_fetch(input);
        assert_eq!(result.method.as_deref(), Some("GET"));
    }

    #[test]
    fn test_real_world_example() {
        let input = r#"fetch("https://anyrouter.top/api/user/models", {
  "headers": {
    "accept": "application/json, text/plain, */*",
    "accept-language": "en,zh-CN;q=0.9,zh;q=0.8",
    "cache-control": "no-store",
    "cookie": "session=MTc2ODgz...; acw_tc=2ff617a2...",
    "Referer": "https://anyrouter.top/console/token"
  },
  "body": null,
  "method": "GET"
});"#;

        let result = parse_fetch(input);

        assert_eq!(result.method.as_deref(), Some("GET"));
        assert_eq!(
            result.url.as_deref(),
            Some("https://anyrouter.top/api/user/models")
        );
        assert_eq!(result.headers.len(), 5);

        // Cookie children
        let cookie = result
            .headers
            .iter()
            .find(|h| h.key == "cookie")
            .unwrap();
        let children = cookie.children.as_ref().unwrap();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].key, "session");
        assert_eq!(children[0].value, "MTc2ODgz...");
    }

    #[test]
    fn test_no_semicolon() {
        let input = r#"fetch("https://example.com/api", {
  "headers": {},
  "body": null,
  "method": "GET"
})"#;

        let result = parse_fetch(input);
        assert_eq!(result.method.as_deref(), Some("GET"));
        assert_eq!(result.url.as_deref(), Some("https://example.com/api"));
    }

    #[test]
    fn test_invalid_input() {
        let result = parse_fetch("not a fetch call");
        assert!(matches!(result.content_type, HttpContentType::Unknown));
        assert!(result.method.is_none());
    }
}
