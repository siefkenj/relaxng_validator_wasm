use relaxng_validator_core::{compile_from_vfs_json, validate_with_vfs_json, ValidationError};
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

// ---------------------------------------------------------------------------
// Owned token types
//
// These mirror src/xmlparser_serde.rs but use owned Strings instead of
// borrowed StrSpan<'a>.  Because they derive Serialize normally (not via a
// serde_json::Value intermediate), serde_wasm_bindgen serializes them as
// plain JS objects rather than as JS Map objects.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
pub struct WasmStrSpan {
    pub text: String,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
pub enum WasmElementEnd {
    Open,
    Close {
        prefix: WasmStrSpan,
        local: WasmStrSpan,
    },
    Empty,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
pub enum WasmExternalId {
    System(WasmStrSpan),
    Public(WasmStrSpan, WasmStrSpan),
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
pub enum WasmEntityDefinition {
    EntityValue(WasmStrSpan),
    ExternalId(WasmExternalId),
}

/// Owned representation of an xmlparser token, suitable for WASM serialization.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(tag = "type")]
pub enum WasmToken {
    Declaration {
        version: WasmStrSpan,
        encoding: Option<WasmStrSpan>,
        standalone: Option<bool>,
        span: WasmStrSpan,
    },
    ProcessingInstruction {
        target: WasmStrSpan,
        content: Option<WasmStrSpan>,
        span: WasmStrSpan,
    },
    Comment {
        text: WasmStrSpan,
        span: WasmStrSpan,
    },
    DtdStart {
        name: WasmStrSpan,
        external_id: Option<WasmExternalId>,
        span: WasmStrSpan,
    },
    EmptyDtd {
        name: WasmStrSpan,
        external_id: Option<WasmExternalId>,
        span: WasmStrSpan,
    },
    EntityDeclaration {
        name: WasmStrSpan,
        definition: WasmEntityDefinition,
        span: WasmStrSpan,
    },
    DtdEnd {
        span: WasmStrSpan,
    },
    ElementStart {
        prefix: WasmStrSpan,
        local: WasmStrSpan,
        span: WasmStrSpan,
    },
    Attribute {
        prefix: WasmStrSpan,
        local: WasmStrSpan,
        value: WasmStrSpan,
        span: WasmStrSpan,
    },
    ElementEnd {
        end: WasmElementEnd,
        span: WasmStrSpan,
    },
    Text {
        text: WasmStrSpan,
    },
    Cdata {
        text: WasmStrSpan,
        span: WasmStrSpan,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    // Smoke test: verify the WASM API types and functions compile correctly in a
    // non-wasm context. Functional validation is covered by the TypeScript tests.
    #[test]
    fn compile_from_vfs_json_returns_compiled_validator() {
        let v = compile_from_vfs_json(r#"{"main.rnc": "start = element root { text }" }"#);
        let result =
            WasmValidator { inner: v }.validate(r#"<?xml version="1.0"?><root>hello</root>"#);
        assert!(result.errors.is_empty());
    }
}

// ---------------------------------------------------------------------------
// Exported types
// ---------------------------------------------------------------------------

/// A single RELAX NG validation error.
#[derive(Debug, Clone, Serialize, Tsify)]
#[tsify(into_wasm_abi)]
#[serde(tag = "type")]
pub enum WasmValidationError {
    /// XML tokenizer or parser error.
    Xml { message: String },
    /// A token that is not allowed at this point in the grammar.
    NotAllowed {
        /// The XML parser token that was not allowed.
        token: WasmToken,
        /// Element names that are valid at this location.
        expected_elements: Vec<String>,
        /// Attribute names that are valid on the current element.
        expected_attributes: Vec<String>,
    },
    /// A namespace prefix was used without being declared.
    UndefinedNamespacePrefix {
        /// Span info for the undefined prefix.
        prefix: WasmStrSpan,
    },
    /// An entity reference names an entity that is not defined.
    UndefinedEntity {
        name: String,
        start: usize,
        end: usize,
    },
    /// An entity is syntactically invalid or not closed.
    InvalidOrUnclosedEntity { start: usize, end: usize },
    /// The internal text buffer limit was exceeded.
    TextBufferOverflow,
    /// The internal pattern limit was exceeded.
    TooManyPatterns,
}

impl From<ValidationError> for WasmValidationError {
    fn from(e: ValidationError) -> Self {
        match e {
            ValidationError::Xml { message } => WasmValidationError::Xml { message },
            ValidationError::NotAllowed {
                token,
                expected_elements,
                expected_attributes,
            } => WasmValidationError::NotAllowed {
                token: serde_json::from_value(token)
                    .expect("token value must deserialize into WasmToken"),
                expected_elements,
                expected_attributes,
            },
            ValidationError::UndefinedNamespacePrefix { prefix } => {
                WasmValidationError::UndefinedNamespacePrefix {
                    prefix: serde_json::from_value(prefix)
                        .expect("prefix value must deserialize into WasmStrSpan"),
                }
            }
            ValidationError::UndefinedEntity { name, span } => {
                WasmValidationError::UndefinedEntity {
                    name,
                    start: span.start,
                    end: span.end,
                }
            }
            ValidationError::InvalidOrUnclosedEntity { span } => {
                WasmValidationError::InvalidOrUnclosedEntity {
                    start: span.start,
                    end: span.end,
                }
            }
            ValidationError::TextBufferOverflow => WasmValidationError::TextBufferOverflow,
            ValidationError::TooManyPatterns => WasmValidationError::TooManyPatterns,
        }
    }
}

/// The result of a validation run.
///
/// `errors` is empty when the document is valid.
#[derive(Debug, Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct ValidationResult {
    pub errors: Vec<WasmValidationError>,
}

impl ValidationResult {
    fn ok() -> Self {
        ValidationResult { errors: vec![] }
    }

    fn from_errors(errors: Vec<ValidationError>) -> Self {
        ValidationResult {
            errors: errors.into_iter().map(WasmValidationError::from).collect(),
        }
    }
}

// ---------------------------------------------------------------------------
// WasmValidator — compiled grammar that can be reused across validations
// ---------------------------------------------------------------------------

/// A compiled RelaxNG grammar that can validate XML documents repeatedly.
///
/// Construct with [`compile_validator`].
#[wasm_bindgen]
pub struct WasmValidator {
    inner: relaxng_validator_core::CompiledValidator,
}

#[wasm_bindgen]
impl WasmValidator {
    /// Validate an XML string against the compiled grammar.
    ///
    /// Returns a [`ValidationResult`] whose `errors` array is empty when valid.
    pub fn validate(&self, doc: &str) -> ValidationResult {
        match self.inner.validate(doc) {
            Ok(()) => ValidationResult::ok(),
            Err(errors) => ValidationResult::from_errors(errors),
        }
    }
}

// ---------------------------------------------------------------------------
// Free functions
// ---------------------------------------------------------------------------

/// Compile a RelaxNG grammar from a JSON virtual file system.
///
/// The JSON object maps file paths to their contents (string or byte array).
/// The **first key** in the object is used as the grammar entry point.
///
/// Returns a [`WasmValidator`] that can validate multiple documents efficiently.
///
/// # Panics
///
/// Panics if the JSON is invalid or the grammar fails to compile.
#[wasm_bindgen]
pub fn compile_validator(vfs_json: &str) -> WasmValidator {
    WasmValidator {
        inner: compile_from_vfs_json(vfs_json),
    }
}

/// Compile a grammar and validate a single XML document in one call.
///
/// The JSON object maps file paths to their contents (string or byte array).
/// The **first key** in the object is used as the grammar entry point.
///
/// Returns a [`ValidationResult`] whose `errors` array is empty when valid.
///
/// # Panics
///
/// Panics if the JSON is invalid or the grammar fails to compile.
#[wasm_bindgen]
pub fn validate(vfs_json: &str, doc: &str) -> ValidationResult {
    match validate_with_vfs_json(vfs_json, doc) {
        Ok(()) => ValidationResult::ok(),
        Err(errors) => ValidationResult::from_errors(errors),
    }
}
