use crate::validation_types::ValidationError;
use std::collections::HashSet;

/// Extracts the best-effort `(start, end)` byte span for a serialized token.
///
/// The token payload may store offsets under either `span` or `text`, depending
/// on token kind.
fn token_span(token: &serde_json::Value) -> Option<(usize, usize)> {
    for field in ["span", "text"] {
        if let Some(obj) = token.get(field).and_then(|v| v.as_object()) {
            let start = obj.get("start")?.as_u64()? as usize;
            let end = obj.get("end")?.as_u64()? as usize;
            return Some((start, end));
        }
    }
    None
}

/// Reduces noisy cascades while retaining distinct, informative errors.
///
/// Behavior:
/// - Keeps informative `NotAllowed` errors (those with expected elements/attrs)
/// - Drops non-informative `Text` and `ElementEnd` `NotAllowed` cascades
/// - Deduplicates exact `(token_type, start, end)` repeats
/// - Preserves non-`NotAllowed` errors untouched
pub fn trim_redundant_errors(errors: Vec<ValidationError>) -> Vec<ValidationError> {
    let mut informative_spans = HashSet::new();
    for err in &errors {
        if let ValidationError::NotAllowed {
            token,
            expected_elements,
            expected_attributes,
        } = err
        {
            if (!expected_elements.is_empty() || !expected_attributes.is_empty())
                && token_span(token).is_some()
            {
                informative_spans.insert(token_span(token).unwrap());
            }
        }
    }

    let mut seen_keys = HashSet::new();
    let mut kept = Vec::new();
    for err in errors {
        match err {
            ValidationError::NotAllowed {
                token,
                expected_elements,
                expected_attributes,
            } => {
                let token_type = token
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string();
                let span = token_span(&token);
                let informative = !expected_elements.is_empty() || !expected_attributes.is_empty();

                // Most post-failure cascades are text/end-tag NotAllowed events.
                if !informative && (token_type == "Text" || token_type == "ElementEnd") {
                    continue;
                }

                if let Some((start, end)) = span {
                    // If an informative error exists at this location, suppress
                    // non-informative duplicates for the same span.
                    if !informative && informative_spans.contains(&(start, end)) {
                        continue;
                    }
                    let key = (token_type.clone(), start, end);
                    if !seen_keys.insert(key) {
                        continue;
                    }
                }

                kept.push(ValidationError::NotAllowed {
                    token,
                    expected_elements,
                    expected_attributes,
                });
            }
            other => kept.push(other),
        }
    }
    kept
}
