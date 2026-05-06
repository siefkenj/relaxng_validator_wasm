use super::{ValidationError, check_simple};

fn not_allowed_errors(schema: &str, doc: &str) -> Vec<ValidationError> {
    check_simple(schema, doc)
        .unwrap_err()
        .into_iter()
        .filter(|e| matches!(e, ValidationError::NotAllowed { .. }))
        .collect()
}

// ---------------------------------------------------------------------------
// Expected elements
// ---------------------------------------------------------------------------

#[test]
fn expected_elements_listed_on_wrong_element() {
    let schema = r#"start = element root { element foo { text } | element bar { text } }"#;
    let doc = r#"<?xml version="1.0"?><root><baz/></root>"#;

    let errs = not_allowed_errors(schema, doc);
    assert!(!errs.is_empty(), "expected at least one NotAllowed error");

    let ValidationError::NotAllowed { expected_elements, .. } = &errs[0] else {
        panic!("expected NotAllowed");
    };

    assert!(
        expected_elements.contains(&"foo".to_string()) && expected_elements.contains(&"bar".to_string()),
        "expected 'foo' and 'bar' in expected_elements, got: {expected_elements:?}"
    );
}

#[test]
fn expected_elements_empty_when_content_is_text() {
    let schema = r#"start = element root { text }"#;
    let doc = r#"<?xml version="1.0"?><root><unexpected/></root>"#;

    let errs = not_allowed_errors(schema, doc);
    assert!(!errs.is_empty());

    let ValidationError::NotAllowed { expected_elements, .. } = &errs[0] else {
        panic!("expected NotAllowed");
    };
    assert!(expected_elements.is_empty(), "expected no expected_elements, got: {expected_elements:?}");
}

// ---------------------------------------------------------------------------
// Expected attributes
// ---------------------------------------------------------------------------

#[test]
fn expected_attributes_listed_on_bad_attribute() {
    let schema = r#"start = element book { attribute isbn { text }, attribute year { text }?, element title { text } }"#;
    // `bad-attr` is not in the schema; the validator should reject it.
    let doc = r#"<?xml version="1.0"?><book bad-attr="x"><title>Hi</title></book>"#;

    let errs = not_allowed_errors(schema, doc);
    assert!(!errs.is_empty(), "expected at least one NotAllowed error");

    let attr_err = errs.iter().find(|e| {
        if let ValidationError::NotAllowed { token, .. } = e {
            token["type"] == "Attribute"
        } else {
            false
        }
    }).expect("expected a NotAllowed error on an Attribute token");

    let ValidationError::NotAllowed { expected_attributes, .. } = attr_err else {
        panic!("expected NotAllowed");
    };

    assert!(
        expected_attributes.contains(&"isbn".to_string()) && expected_attributes.contains(&"year".to_string()),
        "expected 'isbn' and 'year' in expected_attributes, got: {expected_attributes:?}"
    );
}

#[test]
fn expected_attributes_empty_for_element_errors() {
    let schema = r#"start = element root { element child { text } }"#;
    let doc = r#"<?xml version="1.0"?><root><wrong/></root>"#;

    let errs = not_allowed_errors(schema, doc);
    assert!(!errs.is_empty());

    let el_err = errs.iter().find(|e| {
        if let ValidationError::NotAllowed { token, .. } = e {
            token["type"] == "ElementStart"
        } else {
            false
        }
    }).expect("expected a NotAllowed error on an ElementStart token");

    let ValidationError::NotAllowed { expected_attributes, .. } = el_err else {
        panic!("expected NotAllowed");
    };
    assert!(expected_attributes.is_empty(), "expected no expected_attributes, got: {expected_attributes:?}");
}

#[test]
fn valid_document_returns_ok() {
    let schema = r#"start = element root { attribute id { text }, element child { text } }"#;
    let doc = r#"<?xml version="1.0"?><root id="x"><child>hello</child></root>"#;
    assert!(check_simple(schema, doc).is_ok());
}
