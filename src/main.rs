use relaxng_validator::hello_world;

fn main() {
    // Safety: hello_world() returns a valid null-terminated static string.
    let msg = unsafe {
        let ptr = hello_world();
        std::ffi::CStr::from_ptr(ptr as *const i8)
            .to_str()
            .expect("hello_world returned invalid UTF-8")
    };
    println!("{}", msg);
}
