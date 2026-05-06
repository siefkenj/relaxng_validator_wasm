# relaxng-validator

A Rust library and CLI tool for RelaxNG validation, compilable to WebAssembly.

## Building

### Native (CLI)

```sh
cargo build --release
cargo run
```

### WebAssembly

Add the wasm32 target if you haven't already:

```sh
rustup target add wasm32-unknown-unknown
```

Build:

```sh
cargo build --release --target wasm32-unknown-unknown
```

The compiled `.wasm` file will be at:

```
target/wasm32-unknown-unknown/release/relaxng_validator.wasm
```

## Usage

### CLI

```sh
./target/release/relaxng-validator
```

### WASM (JavaScript example)

```js
const { instance } = await WebAssembly.instantiateStreaming(fetch("relaxng_validator.wasm"));
const ptr = instance.exports.hello_world();
// ptr is a pointer into WASM linear memory; decode as a null-terminated UTF-8 string
```

## License

TODO
