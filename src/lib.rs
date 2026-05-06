mod xmlparser_serde;

use relaxng_model::{Compiler, Files, RelaxError, Syntax};
use relaxng_validator_lib::{Validator, ValidatorError};
use serde_json::json;
use std::io;
use std::path::Path;
use xmlparser_serde::SerToken;

fn ser_error(err: ValidatorError<'_>) -> serde_json::Value {
    match err {
        ValidatorError::Xml(e) => json!({
            "type": "Xml",
            "message": e.to_string(),
        }),
        ValidatorError::NotAllowed(token) => {
            let ser = SerToken::from(token);
            json!({
                "type": "NotAllowed",
                "token": serde_json::to_value(&ser).unwrap(),
            })
        }
        ValidatorError::UndefinedNamespacePrefix { prefix } => {
            json!({
                "type": "UndefinedNamespacePrefix",
                "prefix": serde_json::to_value(&xmlparser_serde::SerStrSpan::from(prefix)).unwrap(),
            })
        }
        ValidatorError::UndefinedEntity { name, span } => json!({
            "type": "UndefinedEntity",
            "name": name,
            "span": { "start": span.start, "end": span.end },
        }),
        ValidatorError::InvalidOrUnclosedEntity { span } => json!({
            "type": "InvalidOrUnclosedEntity",
            "span": { "start": span.start, "end": span.end },
        }),
        ValidatorError::TextBufferOverflow => json!({ "type": "TextBufferOverflow" }),
        ValidatorError::TooManyPatterns => json!({ "type": "TooManyPatterns" }),
    }
}

/// Validates `doc` (XML string) against `schema` (RNC compact syntax string).
/// Returns `Ok(())` if valid, or `Err(json)` where the error is a JSON object:
/// `{ "errors": [...] }`
pub fn check_simple(schema: &str, doc: &str) -> Result<(), String> {
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
    let schema = match c.compile(input) {
        Ok(s) => s,
        Err(e) => {
            c.dump_diagnostic(&e);
            panic!("{e:?}");
        }
    };

    let reader = xmlparser::Tokenizer::from(doc);
    let mut v = Validator::new(schema.start, reader).unwrap();
    let mut errors = Vec::new();
    while let Some(i) = v.validate_next() {
        if let Err(err) = i {
            errors.push(ser_error(err));
        }
    }
    if !errors.is_empty() {
        return Err(json!({ "errors": errors }).to_string());
    }
    Ok(())
}

/// Validates a trivial schema and returns a greeting message.
pub fn greet() -> String {
    let schema = "start = element hello { text }";
    let doc = "<?xml version=\"1.0\"?><hello>world\n<b><c/></b></hello>";
    match check_simple(schema, doc) {
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
