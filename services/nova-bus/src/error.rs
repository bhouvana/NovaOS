use crate::proto::ErrorCode;

/// Client-side error type. Mirrors `ErrorCode` from
/// docs/specs/15-NOVA-BUS-PROTOCOL-SPEC.md §11, plus transport-level failures
/// that never reach the wire (connection loss, framing violations).
#[derive(Debug, thiserror::Error)]
pub enum BusError {
    #[error("permission denied for topic {topic}")]
    PermissionDenied { topic: String },
    #[error("call to {topic} timed out")]
    Timeout { topic: String },
    #[error("no handler registered for topic {topic}")]
    NoHandler { topic: String },
    #[error("a handler is already registered for topic {topic}")]
    HandlerAlreadyRegistered { topic: String },
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
    #[error("topic not found: {0}")]
    TopicNotFound(String),
    #[error("protocol version unsupported")]
    ProtocolVersionUnsupported,
    #[error("internal error: {0}")]
    Internal(String),
    #[error("transport error: {0}")]
    Transport(#[from] std::io::Error),
    #[error("connection closed")]
    ConnectionClosed,
    #[error("message exceeds MAX_MESSAGE_SIZE ({0} bytes)")]
    MessageTooLarge(usize),
}

impl BusError {
    pub fn from_wire(code: i32, topic_hint: &str, message: String) -> Self {
        match ErrorCode::try_from(code).unwrap_or(ErrorCode::Unknown) {
            ErrorCode::PermissionDenied => BusError::PermissionDenied {
                topic: topic_hint.to_string(),
            },
            ErrorCode::Timeout => BusError::Timeout {
                topic: topic_hint.to_string(),
            },
            ErrorCode::NoHandler => BusError::NoHandler {
                topic: topic_hint.to_string(),
            },
            ErrorCode::HandlerAlreadyRegistered => BusError::HandlerAlreadyRegistered {
                topic: topic_hint.to_string(),
            },
            ErrorCode::InvalidArgument => BusError::InvalidArgument(message),
            ErrorCode::TopicNotFound => BusError::TopicNotFound(topic_hint.to_string()),
            ErrorCode::ProtocolVersionUnsupported => BusError::ProtocolVersionUnsupported,
            ErrorCode::Internal | ErrorCode::Unknown => BusError::Internal(message),
        }
    }
}
