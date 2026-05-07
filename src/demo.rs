use crate::validation::check_with_json_return;

/// Runs a built-in demo validation and returns a human-readable status string.
pub fn greet() -> String {
    let schema = include_str!("assets/pretext.rnc");
    let doc = "<?xml version=\"1.0\"?>
    <pretext>
        <article xml:id=\"hello-world\" ab='c'>
        <title>Hi there</title>
        <p>Hello, World!</p>
        </article>
    </pretext>
    ";
    match check_with_json_return(schema, doc) {
        Ok(()) => "Hello, world! (schema validated)".to_string(),
        Err(x) => format!("Hello, world! (validation failed: {x})"),
    }
}

/// WASM-exported entry point that prints the demo result.
#[no_mangle]
pub extern "C" fn hello_world() {
    println!("{}", greet());
}
