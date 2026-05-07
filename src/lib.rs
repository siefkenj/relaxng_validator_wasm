mod error_filter;
mod expected_attrs;
mod validation;
mod validation_types;
mod vfs;
mod xmlparser_serde;

pub use validation::{
    check_simple, check_with_json_return, compile_from_vfs_json, validate_with_vfs_json,
    CompiledValidator,
};
pub use validation_types::{SpanInfo, ValidationError};
pub use vfs::{VfsFileContent, VirtualFileSystem};

#[cfg(test)]
#[path = "lib.test.rs"]
mod tests;
