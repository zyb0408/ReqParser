use serde::{Deserialize, Serialize};

/// 解析树中的单个键值节点。
/// 对应 Tech.md 中的 ParseNode 接口。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParseNode {
    pub key: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<ParseNode>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// 检测到的 HTTP 内容类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HttpContentType {
    Request,
    Response,
    HeadersOnly,
    Unknown,
}

/// 解析引擎返回的完整结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParseResult {
    pub content_type: HttpContentType,
    pub method: Option<String>,
    pub url: Option<String>,
    pub status_code: Option<u16>,
    pub status_text: Option<String>,
    pub protocol: Option<String>,
    pub headers: Vec<ParseNode>,
    pub query_params: Option<Vec<ParseNode>>,
    pub body: Option<String>,
    pub raw_text: String,
}
