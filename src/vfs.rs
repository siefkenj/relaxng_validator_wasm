use indexmap::IndexMap;
use relaxng_model::{Files, RelaxError};
use serde::Deserialize;
use std::io;
use std::path::Path;

/// The content of a file in a [`VirtualFileSystem`]: either UTF-8 text or raw bytes.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum VfsFileContent {
    Text(String),
    Bytes(Vec<u8>),
}

/// An in-memory file system that can be deserialized from a JSON object.
///
/// # JSON format
///
/// ```json
/// {
///   "main.rnc": "start = element root { text }",
///   "binary.rnc": [115, 116, 97, 114, 116]
/// }
/// ```
///
/// Keys are file paths; values are either a UTF-8 string or an array of bytes.
#[derive(Debug, Deserialize)]
pub struct VirtualFileSystem(IndexMap<String, VfsFileContent>);

impl VirtualFileSystem {
    /// Create a VFS with a single file at `path` containing `content`.
    pub fn from_single(path: impl Into<String>, content: impl Into<String>) -> Self {
        let mut map = IndexMap::new();
        map.insert(path.into(), VfsFileContent::Text(content.into()));
        VirtualFileSystem(map)
    }

    /// Create a VFS from a map of path → text content.
    /// Note: `std::collections::HashMap` does not preserve insertion order; consider
    /// building the map with [`indexmap::IndexMap`] if order matters.
    pub fn from_map(map: std::collections::HashMap<String, String>) -> Self {
        VirtualFileSystem(
            map.into_iter()
                .map(|(k, v)| (k, VfsFileContent::Text(v)))
                .collect(),
        )
    }

    /// Returns the first key in insertion order, or `None` if the VFS is empty.
    pub fn first_key(&self) -> Option<&str> {
        self.0.keys().next().map(|s| s.as_str())
    }
}

impl Files for VirtualFileSystem {
    fn load(&self, name: &Path) -> Result<String, RelaxError> {
        let key = name.to_string_lossy();
        match self.0.get(key.as_ref()) {
            Some(VfsFileContent::Text(s)) => Ok(s.clone()),
            Some(VfsFileContent::Bytes(b)) => String::from_utf8(b.clone()).map_err(|e| {
                RelaxError::Io(
                    name.to_path_buf(),
                    io::Error::new(io::ErrorKind::InvalidData, e),
                )
            }),
            None => Err(RelaxError::Io(
                name.to_path_buf(),
                io::Error::from(io::ErrorKind::NotFound),
            )),
        }
    }
}

#[cfg(test)]
#[path = "vfs.test.rs"]
mod tests;
