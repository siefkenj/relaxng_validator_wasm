mod expected_attrs;
mod xmlparser_serde;

use relaxng_model::{Compiler, Files, RelaxError, Syntax};
use relaxng_validator_lib::{Validator, ValidatorError};
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io;
use std::path::Path;
use xmlparser_serde::SerToken;

/// A single structured validation error, returned by `check_simple`.
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum ValidationError {
    Xml {
        message: String,
    },
    NotAllowed {
        token: serde_json::Value,
        expected_elements: Vec<String>,
        expected_attributes: Vec<String>,
    },
    UndefinedNamespacePrefix {
        prefix: serde_json::Value,
    },
    UndefinedEntity {
        name: String,
        span: SpanInfo,
    },
    InvalidOrUnclosedEntity {
        span: SpanInfo,
    },
    TextBufferOverflow,
    TooManyPatterns,
}

#[derive(Debug, Serialize)]
pub struct SpanInfo {
    pub start: usize,
    pub end: usize,
}

/// Extract expected elements from a `NotAllowed` error by calling the
/// validator's public `diagnostic()` method and parsing its Help message.
/// The `describe_expected` inside the library only lists elements; attribute
/// extraction is handled separately via `expected_attrs::find_expected_attrs`.
fn extract_expected_elements(v: &Validator<'_>, doc: &str, err: &ValidatorError<'_>) -> Vec<String> {
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
            span: SpanInfo { start: span.start, end: span.end },
        },
        ValidatorError::InvalidOrUnclosedEntity { span } => ValidationError::InvalidOrUnclosedEntity {
            span: SpanInfo { start: span.start, end: span.end },
        },
        ValidatorError::TextBufferOverflow => ValidationError::TextBufferOverflow,
        ValidatorError::TooManyPatterns => ValidationError::TooManyPatterns,
    }
}

/// Validates `doc` (XML string) against `schema` (RNC compact syntax string).
/// Returns `Ok(())` if valid, or `Err(errors)` with a list of structured errors.
pub fn check_simple(schema: &str, doc: &str) -> Result<(), Vec<ValidationError>> {
    struct FS(String);
    impl Files for FS {
        fn load(&self, name: &Path) -> Result<String, RelaxError> {
            match name.to_str().unwrap() {
                "main.rnc" => Ok(self.0.clone()),
                _ => Err(RelaxError::Io(
                    name.to_path_buf(),
                    io::Error::from(io::ErrorKind::NotFound),
                )),
            }
        }
    }

    let mut c = Compiler::new(FS(schema.to_string()), Syntax::Compact);
    let input = Path::new("main.rnc");
    let compiled = match c.compile(input) {
        Ok(s) => s,
        Err(e) => {
            c.dump_diagnostic(&e);
            panic!("{e:?}");
        }
    };

    // Clone the Rc before it is moved into the Validator so we can still
    // access the schema model for expected-attribute lookups.
    let schema_model = compiled.start.clone();

    // Pre-scan the document to build a map from each Attribute token's span
    // start offset to the local name of the element it belongs to.
    // This lets us resolve "which element were we opening?" for attribute errors.
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
                // Attribute rejected: ask the schema model what attributes the
                // current element accepts (expected_elements is N/A here).
                ValidatorError::NotAllowed(xmlparser::Token::Attribute { span, .. }) => {
                    let el_local = attr_span_to_element
                        .get(&span.start())
                        .map(|s| s.as_str())
                        .unwrap_or("");
                    let attrs = {
                        let borrowed = schema_model.borrow();
                        if let Some(dr) = borrowed.as_ref() {
                            let mut visited = HashSet::new();
                            expected_attrs::find_expected_attrs(dr.pattern(), el_local, &mut visited)
                        } else {
                            vec![]
                        }
                    };
                    (vec![], attrs)
                }
                // Other NotAllowed: use the diagnostic Help message for elements.
                ValidatorError::NotAllowed(_) => {
                    (extract_expected_elements(&v, doc, &err), vec![])
                }
                _ => (vec![], vec![]),
            };
            errors.push(to_validation_error(err, expected_elements, expected_attributes));
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(())
}

/// Like `check_simple`, but serialises errors to a JSON string:
/// `{ "errors": [...] }`
pub fn check_with_json_return(schema: &str, doc: &str) -> Result<(), String> {
    check_simple(schema, doc).map_err(|errors| {
        json!({ "errors": errors }).to_string()
    })
}

/// Validates a trivial schema and returns a greeting message.
pub fn greet() -> String {
    // let schema = "start = element hello { text }";
    let schema = include_str!("assets/pretext.rnc");
    let doc = "<?xml version=\"1.0\"?>
    <pretext>
        <article xml:id=\"hello-world\" ab='c'>
        <title>Hi there</title>
        <p>Hello, World!</p>
        </article>
    </pretext>
    ";
    match check_with_json_return(schema, doc) {
        Ok(()) => "Hello, world! (schema validated)".to_string(),
        Err(x) => format!("Hello, world! (validation failed: {x})"),
    }
}

/// WASM-exported entry point. Prints the greeting to stdout.
/// For WASM hosts that need a pointer, use a language-specific binding instead.
#[no_mangle]
pub extern "C" fn hello_world() {
    println!("{}", greet());
}

#[cfg(test)]
#[path = "lib.test.rs"]
mod tests;
