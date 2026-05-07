use super::{VfsFileContent, VirtualFileSystem};
use relaxng_model::Files;
use std::path::Path;

use serde_json_wasm::from_str;

// ---------------------------------------------------------------------------
// from_single
// ---------------------------------------------------------------------------

#[test]
fn from_single_loads_the_file() {
    let vfs = VirtualFileSystem::from_single("schema.rnc", "start = text");
    let result = vfs.load(Path::new("schema.rnc")).unwrap();
    assert_eq!(result, "start = text");
}

#[test]
fn from_single_missing_file_returns_not_found() {
    let vfs = VirtualFileSystem::from_single("schema.rnc", "start = text");
    let err = vfs.load(Path::new("other.rnc")).unwrap_err();
    let relaxng_model::RelaxError::Io(path, io_err) = err else {
        panic!("expected RelaxError::Io");
    };
    assert_eq!(path, std::path::PathBuf::from("other.rnc"));
    assert_eq!(io_err.kind(), std::io::ErrorKind::NotFound);
}

// ---------------------------------------------------------------------------
// JSON deserialization
// ---------------------------------------------------------------------------

#[test]
fn deserialize_string_values() {
    let json = r#"{ "a.rnc": "start = text", "b.rnc": "start = empty" }"#;
    let vfs: VirtualFileSystem = from_str(json).unwrap();
    assert_eq!(vfs.load(Path::new("a.rnc")).unwrap(), "start = text");
    assert_eq!(vfs.load(Path::new("b.rnc")).unwrap(), "start = empty");
}

#[test]
fn deserialize_byte_array_values() {
    // "hello" as UTF-8 bytes
    let json = r#"{ "file.rnc": [104, 101, 108, 108, 111] }"#;
    let vfs: VirtualFileSystem = from_str(json).unwrap();
    assert_eq!(vfs.load(Path::new("file.rnc")).unwrap(), "hello");
}

#[test]
fn deserialize_mixed_string_and_bytes() {
    let json = r#"{ "text.rnc": "start = text", "bytes.rnc": [116, 101, 120, 116] }"#;
    let vfs: VirtualFileSystem = from_str(json).unwrap();
    assert_eq!(vfs.load(Path::new("text.rnc")).unwrap(), "start = text");
    assert_eq!(vfs.load(Path::new("bytes.rnc")).unwrap(), "text");
}

#[test]
fn deserialize_empty_object_gives_empty_vfs() {
    let vfs: VirtualFileSystem = from_str("{}").unwrap();
    assert!(vfs.load(Path::new("anything")).is_err());
}

// ---------------------------------------------------------------------------
// Files trait — load behaviour
// ---------------------------------------------------------------------------

#[test]
fn load_missing_key_returns_not_found() {
    let json = r#"{ "present.rnc": "start = text" }"#;
    let vfs: VirtualFileSystem = from_str(json).unwrap();
    let err = vfs.load(Path::new("absent.rnc")).unwrap_err();
    let relaxng_model::RelaxError::Io(_, io_err) = err else {
        panic!("expected RelaxError::Io");
    };
    assert_eq!(io_err.kind(), std::io::ErrorKind::NotFound);
}

#[test]
fn load_invalid_utf8_bytes_returns_invalid_data() {
    // 0xFF is not valid UTF-8
    let json = r#"{ "bad.rnc": [255, 254] }"#;
    let vfs: VirtualFileSystem = from_str(json).unwrap();
    let err = vfs.load(Path::new("bad.rnc")).unwrap_err();
    let relaxng_model::RelaxError::Io(_, io_err) = err else {
        panic!("expected RelaxError::Io");
    };
    assert_eq!(io_err.kind(), std::io::ErrorKind::InvalidData);
}

#[test]
fn load_empty_string_file() {
    let vfs = VirtualFileSystem::from_single("empty.rnc", "");
    assert_eq!(vfs.load(Path::new("empty.rnc")).unwrap(), "");
}

#[test]
fn load_empty_byte_array_file() {
    let json = r#"{ "empty.rnc": [] }"#;
    let vfs: VirtualFileSystem = from_str(json).unwrap();
    assert_eq!(vfs.load(Path::new("empty.rnc")).unwrap(), "");
}

// ---------------------------------------------------------------------------
// VfsFileContent deserialization
// ---------------------------------------------------------------------------

#[test]
fn vfs_file_content_text_variant() {
    let content: VfsFileContent = from_str(r#""hello""#).unwrap();
    assert!(matches!(content, VfsFileContent::Text(_)));
}

#[test]
fn vfs_file_content_bytes_variant() {
    let content: VfsFileContent = from_str("[1, 2, 3]").unwrap();
    assert!(matches!(content, VfsFileContent::Bytes(_)));
}
