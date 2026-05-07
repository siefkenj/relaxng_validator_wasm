use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use relaxng_validator_core::{compile_from_vfs_json, validate_with_vfs_json, ValidationError};

// ---------------------------------------------------------------------------
// Python-visible error type
// ---------------------------------------------------------------------------

/// A single RELAX NG validation error.
#[pyclass(frozen)]
#[derive(Clone)]
pub struct PyValidationError {
    #[pyo3(get)]
    pub r#type: String,
    #[pyo3(get)]
    pub message: Option<String>,
    /// JSON string of the xmlparser token (for NotAllowed errors).
    #[pyo3(get)]
    pub token: Option<String>,
    #[pyo3(get)]
    pub expected_elements: Option<Vec<String>>,
    #[pyo3(get)]
    pub expected_attributes: Option<Vec<String>>,
    /// JSON string of the prefix info (for UndefinedNamespacePrefix errors).
    #[pyo3(get)]
    pub prefix: Option<String>,
    #[pyo3(get)]
    pub name: Option<String>,
    #[pyo3(get)]
    pub start: Option<usize>,
    #[pyo3(get)]
    pub end: Option<usize>,
}

#[pymethods]
impl PyValidationError {
    fn __repr__(&self) -> String {
        format!("ValidationError(type={:?})", self.r#type)
    }
}

impl From<ValidationError> for PyValidationError {
    fn from(e: ValidationError) -> Self {
        match e {
            ValidationError::Xml { message } => PyValidationError {
                r#type: "Xml".to_string(),
                message: Some(message),
                token: None,
                expected_elements: None,
                expected_attributes: None,
                prefix: None,
                name: None,
                start: None,
                end: None,
            },
            ValidationError::NotAllowed {
                token,
                expected_elements,
                expected_attributes,
            } => PyValidationError {
                r#type: "NotAllowed".to_string(),
                message: None,
                token: Some(
                    serde_json_wasm::to_string(&token).unwrap_or_else(|_| "null".to_string()),
                ),
                expected_elements: Some(expected_elements),
                expected_attributes: Some(expected_attributes),
                prefix: None,
                name: None,
                start: None,
                end: None,
            },
            ValidationError::UndefinedNamespacePrefix { prefix } => PyValidationError {
                r#type: "UndefinedNamespacePrefix".to_string(),
                message: None,
                token: None,
                expected_elements: None,
                expected_attributes: None,
                prefix: Some(
                    serde_json_wasm::to_string(&prefix).unwrap_or_else(|_| "null".to_string()),
                ),
                name: None,
                start: None,
                end: None,
            },
            ValidationError::UndefinedEntity { name, span } => PyValidationError {
                r#type: "UndefinedEntity".to_string(),
                message: None,
                token: None,
                expected_elements: None,
                expected_attributes: None,
                prefix: None,
                name: Some(name),
                start: Some(span.start),
                end: Some(span.end),
            },
            ValidationError::InvalidOrUnclosedEntity { span } => PyValidationError {
                r#type: "InvalidOrUnclosedEntity".to_string(),
                message: None,
                token: None,
                expected_elements: None,
                expected_attributes: None,
                prefix: None,
                name: None,
                start: Some(span.start),
                end: Some(span.end),
            },
            ValidationError::TextBufferOverflow => PyValidationError {
                r#type: "TextBufferOverflow".to_string(),
                message: None,
                token: None,
                expected_elements: None,
                expected_attributes: None,
                prefix: None,
                name: None,
                start: None,
                end: None,
            },
            ValidationError::TooManyPatterns => PyValidationError {
                r#type: "TooManyPatterns".to_string(),
                message: None,
                token: None,
                expected_elements: None,
                expected_attributes: None,
                prefix: None,
                name: None,
                start: None,
                end: None,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Compiled validator wrapper
// ---------------------------------------------------------------------------

/// A compiled RELAX NG grammar that can validate XML documents repeatedly.
///
/// Construct with :func:`compile_validator`.
#[pyclass(unsendable)]
pub struct Validator {
    inner: relaxng_validator_core::CompiledValidator,
}

#[pymethods]
impl Validator {
    /// Validate an XML string against the compiled grammar.
    ///
    /// Returns a list of :class:`ValidationError`. The list is empty when valid.
    pub fn validate(&self, doc: &str) -> Vec<PyValidationError> {
        match self.inner.validate(doc) {
            Ok(()) => vec![],
            Err(errors) => errors.into_iter().map(PyValidationError::from).collect(),
        }
    }
}

// ---------------------------------------------------------------------------
// Module-level functions
// ---------------------------------------------------------------------------

/// Compile a RELAX NG grammar from a JSON virtual file system string.
///
/// The JSON object maps file paths to their contents (``str`` or ``list[int]``).
/// The **first key** in the object is used as the grammar entry point.
///
/// Returns a :class:`Validator` that can validate multiple documents efficiently.
///
/// :raises RuntimeError: If the JSON is invalid or the grammar fails to compile.
#[pyfunction]
pub fn compile_validator(vfs_json: &str) -> PyResult<Validator> {
    // compile_from_vfs_json panics on error; catch it as a Python RuntimeError.
    match std::panic::catch_unwind(|| compile_from_vfs_json(vfs_json)) {
        Ok(inner) => Ok(Validator { inner }),
        Err(_) => Err(PyRuntimeError::new_err(
            "Failed to compile RELAX NG grammar (invalid JSON or grammar error)",
        )),
    }
}

/// Compile a grammar and validate a single XML document in one call.
///
/// The JSON object maps file paths to their contents (``str`` or ``list[int]``).
/// The **first key** in the object is used as the grammar entry point.
///
/// Returns a list of :class:`ValidationError`. The list is empty when valid.
///
/// :raises RuntimeError: If the JSON is invalid or the grammar fails to compile.
#[pyfunction]
pub fn validate(vfs_json: &str, doc: &str) -> PyResult<Vec<PyValidationError>> {
    match std::panic::catch_unwind(|| validate_with_vfs_json(vfs_json, doc)) {
        Ok(Ok(())) => Ok(vec![]),
        Ok(Err(errors)) => Ok(errors.into_iter().map(PyValidationError::from).collect()),
        Err(_) => Err(PyRuntimeError::new_err(
            "Failed to compile RELAX NG grammar (invalid JSON or grammar error)",
        )),
    }
}

// ---------------------------------------------------------------------------
// Module definition
// ---------------------------------------------------------------------------

#[pymodule]
fn relaxng_validator(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyValidationError>()?;
    m.add_class::<Validator>()?;
    m.add_function(wrap_pyfunction!(compile_validator, m)?)?;
    m.add_function(wrap_pyfunction!(validate, m)?)?;
    Ok(())
}
