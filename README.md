# relaxng-validator-wasm

Validate XML with RELAX NG in your browser! Try the [Playground](https://siefkenj.github.io/relaxng_validator_wasm/) at https://siefkenj.github.io/relaxng_validator_wasm/

## Repository layout

- `src/` - core Rust wrapper around the [RelaxNG Rust](https://github.com/dholroyd/relaxng-rust) project.
- `wasm-api/` - WASM and Typescript bindings
- `python-api/` - Python bindings
- `relaxng-rust/` - upstream RelaxNG Rust crates (submodule)

## Subproject guides

- WASM guide: [wasm-api/README.md](wasm-api/README.md)
- Python guide: [python-api/README.md](python-api/README.md)

## Quick start

### Rust build

```sh
cargo build
```

### WASM build and tests

```sh
npm test
```

This rebuilds the WASM package and runs Vitest.

### Python setup and tests

```sh
python3 -m venv .venv
source .venv/bin/activate
pip install pytest maturin
maturin develop
pytest
```

## CLI usage

The CLI expects schema file(s) followed by the XML document path.

Example:

```sh
cargo run -- pretext.rnc test.xml
```
