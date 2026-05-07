mod demo;
mod error_filter;
mod expected_attrs;
mod validation;
mod validation_types;
mod xmlparser_serde;

pub use demo::{greet, hello_world};
pub use validation::{check_simple, check_with_json_return};
pub use validation_types::{SpanInfo, ValidationError};

#[cfg(test)]
#[path = "lib.test.rs"]
mod tests;
