use serde::Serialize;

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum AppError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Clipboard error: {0}")]
    ClipboardError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}
