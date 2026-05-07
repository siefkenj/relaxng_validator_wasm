use crate::error_filter::trim_redundant_errors;
use crate::expected_attrs;
use crate::validation_types::{SpanInfo, ValidationError};
use crate::vfs::VirtualFileSystem;
use crate::xmlparser_serde;
use crate::xmlparser_serde::SerToken;
use relaxng_model::{Compiler, Syntax};
use relaxng_validator_lib::{Validator, ValidatorError};
use serde_json::json;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;

/// Extracts expected element names from validator diagnostics for a `NotAllowed` error.
fn extract_expected_elements(
    v: &Validator<'_>,
    doc: &str,
    err: &ValidatorError<'_>,
) -> Vec<String> {
    let (_, diagnostics) = v.diagnostic("doc".to_string(), doc.to_string(), err);
    diagnostics
        .iter()
        .find(|d| d.level == codemap_diagnostic::Level::Help)
        .and_then(|d| d.message.strip_prefix("Expected Element "))
        .map(|rest| {
            rest.split_whitespace()
                .take_while(|s| !s.starts_with(".."))
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// Converts a library `ValidatorError` into the crate's serializable error type.
fn to_validation_error(
    err: ValidatorError<'_>,
    expected_elements: Vec<String>,
    expected_attributes: Vec<String>,
) -> ValidationError {
    match err {
        ValidatorError::Xml(e) => ValidationError::Xml {
            message: e.to_string(),
        },
        ValidatorError::NotAllowed(token) => ValidationError::NotAllowed {
            token: serde_json::to_value(&SerToken::from(token)).unwrap(),
            expected_elements,
            expected_attributes,
        },
        ValidatorError::UndefinedNamespacePrefix { prefix } => {
            ValidationError::UndefinedNamespacePrefix {
                prefix: serde_json::to_value(&xmlparser_serde::SerStrSpan::from(prefix)).unwrap(),
            }
        }
        ValidatorError::UndefinedEntity { name, span } => ValidationError::UndefinedEntity {
            name: name.to_string(),
            span: SpanInfo {
                start: span.start,
                end: span.end,
            },
        },
        ValidatorError::InvalidOrUnclosedEntity { span } => {
            ValidationError::InvalidOrUnclosedEntity {
                span: SpanInfo {
                    start: span.start,
                    end: span.end,
                },
            }
        }
        ValidatorError::TextBufferOverflow => ValidationError::TextBufferOverflow,
        ValidatorError::TooManyPatterns => ValidationError::TooManyPatterns,
    }
}

/// Validates `doc` (XML) against the schema at `schema_path` within `vfs`.
///
/// Returns:
/// - `Ok(())` if valid
/// - `Err(Vec<ValidationError>)` with filtered, non-redundant errors if invalid
pub fn check_simple(
    vfs: VirtualFileSystem,
    schema_path: &str,
    doc: &str,
) -> Result<(), Vec<ValidationError>> {
    let mut c = Compiler::new(vfs, Syntax::Auto);
    let input = Path::new(schema_path);
    let compiled = match c.compile(input) {
        Ok(s) => s,
        Err(e) => {
            c.dump_diagnostic(&e);
            panic!("{e:?}");
        }
    };

    // Keep a clone of the model for attribute expectation lookup.
    let schema_model = compiled.start.clone();

    // Build a lookup from attribute token span start -> owning element local name.
    let attr_span_to_element: HashMap<usize, String> = {
        let mut map = HashMap::new();
        let mut current_el: Option<String> = None;
        for tok_result in xmlparser::Tokenizer::from(doc) {
            match tok_result {
                Ok(xmlparser::Token::ElementStart { local, .. }) => {
                    current_el = Some(local.as_str().to_string());
                }
                Ok(xmlparser::Token::Attribute { span, .. }) => {
                    if let Some(ref el) = current_el {
                        map.insert(span.start(), el.clone());
                    }
                }
                Ok(xmlparser::Token::ElementEnd { .. }) => {
                    current_el = None;
                }
                _ => {}
            }
        }
        map
    };

    let reader = xmlparser::Tokenizer::from(doc);
    let mut v = Validator::new(compiled.start, reader).unwrap();
    let mut errors = Vec::new();
    while let Some(i) = v.validate_next() {
        if let Err(err) = i {
            let (expected_elements, expected_attributes) = match &err {
                ValidatorError::NotAllowed(xmlparser::Token::Attribute { span, .. }) => {
                    let el_local = attr_span_to_element
                        .get(&span.start())
                        .map(|s| s.as_str())
                        .unwrap_or("");
                    let attrs = {
                        let borrowed = schema_model.borrow();
                        if let Some(dr) = borrowed.as_ref() {
                            let mut visited = HashSet::new();
                            expected_attrs::find_expected_attrs(
                                dr.pattern(),
                                el_local,
                                &mut visited,
                            )
                        } else {
                            vec![]
                        }
                    };
                    (vec![], attrs)
                }
                ValidatorError::NotAllowed(_) => (extract_expected_elements(&v, doc, &err), vec![]),
                _ => (vec![], vec![]),
            };
            errors.push(to_validation_error(
                err,
                expected_elements,
                expected_attributes,
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(trim_redundant_errors(errors))
    }
}

/// Validates like [`check_simple`], but returns a JSON string on failure.
///
/// Error shape: `{ "errors": [...] }`.
pub fn check_with_json_return(schema: &str, doc: &str) -> Result<(), String> {
    let vfs = VirtualFileSystem::from_single("main.rnc", schema);
    check_simple(vfs, "main.rnc", doc).map_err(|errors| json!({ "errors": errors }).to_string())
}
