use url::Url;

use crate::models::{HttpContentType, ParseNode, ParseResult};

/// 将 cURL 命令文本解析为结构化的 ParseResult。
pub fn parse_curl(input: &str) -> ParseResult {
    let raw_text = input.to_string();
    let trimmed = input.trim();

    // 去掉 curl 前缀
    let without_curl = strip_curl_prefix(trimmed);

    // 合并反斜杠续行
    let merged = merge_continuation_lines(without_curl);

    // Shell token 化
    let tokens = shell_tokenize(&merged);

    let mut method: Option<String> = None;
    let mut url_str: Option<String> = None;
    let mut headers: Vec<ParseNode> = Vec::new();
    let mut body: Option<String> = None;
    let mut has_data = false;

    let mut i = 0;
    while i < tokens.len() {
        let tok = &tokens[i];
        match tok.as_str() {
            "-H" | "--header" => {
                if let Some(val) = tokens.get(i + 1) {
                    if let Some(header) = parse_header_token(val) {
                        headers.push(header);
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "-b" | "--cookie" => {
                if let Some(val) = tokens.get(i + 1) {
                    let children = parse_cookie_children(val);
                    headers.push(ParseNode {
                        key: "Cookie".to_string(),
                        value: val.clone(),
                        children,
                        description: None,
                    });
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "-X" | "--request" => {
                if let Some(val) = tokens.get(i + 1) {
                    method = Some(val.to_uppercase());
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "-d" | "--data" | "--data-raw" | "--data-binary" | "--data-urlencode" => {
                if let Some(val) = tokens.get(i + 1) {
                    body = Some(val.clone());
                    has_data = true;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            _ => {
                // 忽略已知的无参数 flag
                if tok == "--compressed" || tok == "--location" || tok == "-L" || tok == "-k"
                    || tok == "--insecure" || tok == "-s" || tok == "--silent"
                    || tok == "-S" || tok == "--show-error" || tok == "-v" || tok == "--verbose"
                {
                    i += 1;
                    continue;
                }
                // 忽略未知的 --flag=value 或 --flag value 形式
                if tok.starts_with("--") && tok.contains('=') {
                    i += 1;
                    continue;
                }
                if tok.starts_with('-') && tok.len() > 1 && url_str.is_some() {
                    // 未知的带参数 flag，跳过它和它的值
                    i += 2;
                    continue;
                }
                // 第一个非 flag token 是 URL
                if url_str.is_none() && !tok.starts_with('-') {
                    url_str = Some(tok.clone());
                }
                i += 1;
            }
        }
    }

    // 默认 method
    let method = method.or_else(|| {
        if has_data {
            Some("POST".to_string())
        } else {
            Some("GET".to_string())
        }
    });

    // 解析 URL query params
    let query_params = url_str.as_ref().and_then(|u| parse_query_params(u));

    ParseResult {
        content_type: HttpContentType::Request,
        method,
        url: url_str,
        status_code: None,
        status_text: None,
        protocol: None,
        headers,
        query_params,
        body,
        raw_text,
    }
}

/// 去掉 `curl ` 前缀（支持 `curl` 后紧跟空格的情况）。
fn strip_curl_prefix(s: &str) -> &str {
    let trimmed = s.trim_start();
    if trimmed.starts_with("curl ") || trimmed.starts_with("curl\t") {
        &trimmed[5..]
    } else if trimmed == "curl" {
        ""
    } else {
        trimmed
    }
}

/// 合并反斜杠续行（`\` + 换行）为单行。
fn merge_continuation_lines(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.peek() {
                Some('\n') => {
                    chars.next();
                    // 跳过续行后的前导空白
                    while let Some(&ws) = chars.peek() {
                        if ws == ' ' || ws == '\t' {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                }
                Some('\r') => {
                    chars.next();
                    if chars.peek() == Some(&'\n') {
                        chars.next();
                    }
                    while let Some(&ws) = chars.peek() {
                        if ws == ' ' || ws == '\t' {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                }
                _ => {
                    result.push(ch);
                }
            }
        } else {
            result.push(ch);
        }
    }
    result
}

/// Shell 风格 token 化。
/// 按空格分割，但尊重单引号、双引号和 `$'...'` 内的内容。
fn shell_tokenize(s: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = s.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' | '\n' | '\r' => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
                chars.next();
            }
            '\'' => {
                chars.next(); // consume opening '
                // 单引号：原样保留内容，直到下一个 '
                while let Some(&c) = chars.peek() {
                    if c == '\'' {
                        chars.next();
                        break;
                    }
                    current.push(c);
                    chars.next();
                }
            }
            '"' => {
                chars.next(); // consume opening "
                // 双引号：处理转义
                while let Some(&c) = chars.peek() {
                    if c == '"' {
                        chars.next();
                        break;
                    }
                    if c == '\\' {
                        chars.next();
                        if let Some(&escaped) = chars.peek() {
                            match escaped {
                                '"' | '\\' | '$' | '`' => {
                                    current.push(escaped);
                                    chars.next();
                                }
                                'n' => {
                                    current.push('\n');
                                    chars.next();
                                }
                                't' => {
                                    current.push('\t');
                                    chars.next();
                                }
                                _ => {
                                    current.push('\\');
                                    current.push(escaped);
                                    chars.next();
                                }
                            }
                        }
                    } else {
                        current.push(c);
                        chars.next();
                    }
                }
            }
            '$' => {
                chars.next();
                if chars.peek() == Some(&'\'') {
                    // $'...' ANSI-C quoting: 按单引号处理
                    chars.next(); // consume '
                    while let Some(&c) = chars.peek() {
                        if c == '\'' {
                            chars.next();
                            break;
                        }
                        current.push(c);
                        chars.next();
                    }
                } else {
                    current.push('$');
                }
            }
            _ => {
                current.push(ch);
                chars.next();
            }
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

/// 解析 `-H 'Key: Value'` 中的 header 字符串为 ParseNode。
fn parse_header_token(header_str: &str) -> Option<ParseNode> {
    let (key, value) = header_str.split_once(':')?;
    let key = key.trim().to_string();
    let value = value.trim().to_string();

    let children = parse_header_value_children(&key, &value);

    Some(ParseNode {
        key,
        value,
        children,
        description: None,
    })
}

/// Cookie 值按 `;` 拆解为 children（与 parser.rs 中的逻辑一致）。
fn parse_cookie_children(cookie_str: &str) -> Option<Vec<ParseNode>> {
    let children: Vec<ParseNode> = cookie_str
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

/// Header value 子解析（用于 -H 传入的 Cookie header）。
fn parse_header_value_children(key: &str, value: &str) -> Option<Vec<ParseNode>> {
    let lower_key = key.to_lowercase();
    if lower_key == "cookie" || lower_key == "set-cookie" {
        parse_cookie_children(value)
    } else {
        None
    }
}

/// 从 URL 字符串中解析查询参数。
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_get_with_headers() {
        let input = "curl 'https://example.com/api/data' \
            -H 'accept: application/json' \
            -H 'user-agent: TestAgent/1.0'";
        let result = parse_curl(input);

        assert_eq!(result.method.as_deref(), Some("GET"));
        assert_eq!(result.url.as_deref(), Some("https://example.com/api/data"));
        assert_eq!(result.headers.len(), 2);
        assert_eq!(result.headers[0].key, "accept");
        assert_eq!(result.headers[0].value, "application/json");
        assert_eq!(result.headers[1].key, "user-agent");
        assert_eq!(result.headers[1].value, "TestAgent/1.0");
    }

    #[test]
    fn test_cookie_with_b_flag() {
        let input = "curl 'https://example.com' -b 'session=abc123; theme=dark; lang=en'";
        let result = parse_curl(input);

        let cookie_header = result.headers.iter().find(|h| h.key == "Cookie").unwrap();
        assert_eq!(cookie_header.value, "session=abc123; theme=dark; lang=en");
        let children = cookie_header.children.as_ref().unwrap();
        assert_eq!(children.len(), 3);
        assert_eq!(children[0].key, "session");
        assert_eq!(children[0].value, "abc123");
        assert_eq!(children[1].key, "theme");
        assert_eq!(children[1].value, "dark");
        assert_eq!(children[2].key, "lang");
        assert_eq!(children[2].value, "en");
    }

    #[test]
    fn test_post_with_data() {
        let input = r#"curl 'https://example.com/api/submit' -X POST -H 'content-type: application/json' --data '{"name":"test","value":42}'"#;
        let result = parse_curl(input);

        assert_eq!(result.method.as_deref(), Some("POST"));
        assert_eq!(
            result.body.as_deref(),
            Some(r#"{"name":"test","value":42}"#)
        );
        assert_eq!(result.headers[0].key, "content-type");
    }

    #[test]
    fn test_url_query_params() {
        let input = "curl 'https://example.com/search?q=hello%20world&lang=zh&page=1'";
        let result = parse_curl(input);

        let params = result.query_params.unwrap();
        assert_eq!(params.len(), 3);
        assert_eq!(params[0].key, "q");
        assert_eq!(params[0].value, "hello world");
        assert_eq!(params[1].key, "lang");
        assert_eq!(params[1].value, "zh");
        assert_eq!(params[2].key, "page");
        assert_eq!(params[2].value, "1");
    }

    #[test]
    fn test_multiline_continuation() {
        let input = "curl 'https://example.com/api' \\\n  -H 'accept: text/html' \\\n  -H 'host: example.com'";
        let result = parse_curl(input);

        assert_eq!(result.method.as_deref(), Some("GET"));
        assert_eq!(result.url.as_deref(), Some("https://example.com/api"));
        assert_eq!(result.headers.len(), 2);
        assert_eq!(result.headers[0].key, "accept");
        assert_eq!(result.headers[1].key, "host");
    }

    #[test]
    fn test_default_method_get() {
        let input = "curl 'https://example.com/'";
        let result = parse_curl(input);
        assert_eq!(result.method.as_deref(), Some("GET"));
    }

    #[test]
    fn test_data_without_x_defaults_post() {
        let input = "curl 'https://example.com/api' -d 'key=value'";
        let result = parse_curl(input);
        assert_eq!(result.method.as_deref(), Some("POST"));
        assert_eq!(result.body.as_deref(), Some("key=value"));
    }

    #[test]
    fn test_data_raw_flag() {
        let input = "curl 'https://example.com/api' --data-raw '{\"a\":1}'";
        let result = parse_curl(input);
        assert_eq!(result.method.as_deref(), Some("POST"));
        assert_eq!(result.body.as_deref(), Some("{\"a\":1}"));
    }

    #[test]
    fn test_double_quoted_url() {
        let input = r#"curl "https://example.com/api?key=val""#;
        let result = parse_curl(input);
        assert_eq!(
            result.url.as_deref(),
            Some("https://example.com/api?key=val")
        );
        let params = result.query_params.unwrap();
        assert_eq!(params[0].key, "key");
        assert_eq!(params[0].value, "val");
    }

    #[test]
    fn test_cookie_via_header() {
        let input = "curl 'https://example.com' -H 'Cookie: sid=xyz; token=abc'";
        let result = parse_curl(input);

        let cookie = result.headers.iter().find(|h| h.key == "Cookie").unwrap();
        let children = cookie.children.as_ref().unwrap();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].key, "sid");
        assert_eq!(children[0].value, "xyz");
    }

    #[test]
    fn test_compressed_flag_ignored() {
        let input = "curl 'https://example.com' --compressed -H 'accept: */*'";
        let result = parse_curl(input);
        assert_eq!(result.headers.len(), 1);
        assert_eq!(result.headers[0].key, "accept");
    }

    #[test]
    fn test_real_example() {
        let input = r#"curl 'https://anyrouter.top/api/log/self/stat?type=0&token_name=&model_name=&start_timestamp=1770480000&end_timestamp=1770536299&group=' \
  -H 'accept: application/json, text/plain, */*' \
  -H 'accept-language: en,zh-CN;q=0.9,zh;q=0.8' \
  -H 'cache-control: no-store' \
  -b 'session=MTc2ODgz; acw_tc=2ff617a2; cdn_sec_tc=2ff617a2; acw_sc__v2=69882ade' \
  -H 'dnt: 1' \
  -H 'new-api-user: 22856' \
  -H 'user-agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)'"#;

        let result = parse_curl(input);

        assert_eq!(result.method.as_deref(), Some("GET"));
        assert!(result.url.as_deref().unwrap().starts_with("https://anyrouter.top/"));

        // query params
        let params = result.query_params.as_ref().unwrap();
        assert_eq!(params[0].key, "type");
        assert_eq!(params[0].value, "0");

        // headers: 6 from -H + 1 from -b (Cookie)
        assert_eq!(result.headers.len(), 7);

        // Cookie children
        let cookie = result.headers.iter().find(|h| h.key == "Cookie").unwrap();
        let children = cookie.children.as_ref().unwrap();
        assert_eq!(children.len(), 4);
        assert_eq!(children[0].key, "session");
        assert_eq!(children[0].value, "MTc2ODgz");
    }

    #[test]
    fn test_empty_query_param_values() {
        let input = "curl 'https://example.com/api?token_name=&model_name='";
        let result = parse_curl(input);

        let params = result.query_params.unwrap();
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].key, "token_name");
        assert_eq!(params[0].value, "");
        assert_eq!(params[1].key, "model_name");
        assert_eq!(params[1].value, "");
    }

    #[test]
    fn test_content_type_is_request() {
        let input = "curl 'https://example.com/'";
        let result = parse_curl(input);
        assert!(matches!(result.content_type, HttpContentType::Request));
    }

    #[test]
    fn test_raw_text_preserved() {
        let input = "curl 'https://example.com/'";
        let result = parse_curl(input);
        assert_eq!(result.raw_text, input);
    }
}
