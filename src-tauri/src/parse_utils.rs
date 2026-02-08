use url::Url;

use crate::models::ParseNode;

/// 从 URL 字符串中解析查询参数。
/// 共享函数，被 parser.rs、curl_parser.rs、fetch_parser.rs 调用。
pub fn parse_query_params(url_str: &str) -> Option<Vec<ParseNode>> {
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
                    decoded_value: None,
                    value_type: None,
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

/// Cookie 值按 `;` 拆解为 children ParseNode 列表。
/// 共享函数，被 parser.rs、curl_parser.rs、fetch_parser.rs 调用。
pub fn parse_cookie_children(cookie_str: &str) -> Option<Vec<ParseNode>> {
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
                    decoded_value: None,
                    value_type: None,
                }
            } else {
                ParseNode {
                    key: pair.to_string(),
                    value: String::new(),
                    children: None,
                    description: None,
                    decoded_value: None,
                    value_type: None,
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

/// 根据 header key 判断是否需要 Cookie 子解析，并返回 children。
pub fn parse_header_value_children(key: &str, value: &str) -> Option<Vec<ParseNode>> {
    let lower_key = key.to_lowercase();
    if lower_key == "cookie" || lower_key == "set-cookie" {
        parse_cookie_children(value)
    } else {
        None
    }
}
