use serde::Serialize;

/// A byte span within the input XML document.
#[derive(Debug, Clone, Serialize)]
pub struct SpanInfo {
    /// Start offset (inclusive).
    pub start: usize,
    /// End offset (exclusive).
    pub end: usize,
}

/// A structured validation error produced by RELAX NG validation.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ValidationError {
    /// XML tokenizer/parser error.
    Xml {
        /// Human-readable parser error message.
        message: String,
    },
    /// Token is not allowed at this point in validation.
    NotAllowed {
        /// Serialized `xmlparser::Token` payload.
        token: serde_json::Value,
        /// Allowed element names at this location.
        expected_elements: Vec<String>,
        /// Allowed attribute names at this location.
        expected_attributes: Vec<String>,
    },
    /// Prefix was used without an in-scope namespace declaration.
    UndefinedNamespacePrefix {
        /// Span/value info for the undefined prefix.
        prefix: serde_json::Value,
    },
    /// Entity reference names an entity that is not defined.
    UndefinedEntity {
        /// Undefined entity name.
        name: String,
        /// Span of the entity usage.
        span: SpanInfo,
    },
    /// Entity syntax is invalid or the entity is not closed.
    InvalidOrUnclosedEntity {
        /// Span of the malformed entity.
        span: SpanInfo,
    },
    /// Internal text buffer limit was exceeded.
    TextBufferOverflow,
    /// Internal pattern limit was exceeded.
    TooManyPatterns,
}
