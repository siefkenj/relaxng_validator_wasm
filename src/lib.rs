/// Returns a hello world greeting string.
///
/// When compiled to WASM, this function is exported and callable from the host
/// (JavaScript, Python, etc.) via the module's exported function table.
#[no_mangle]
pub extern "C" fn hello_world() -> *const u8 {
    b"Hello, world!\0".as_ptr()
}
