# relaxng-validator

RELAX NG validation toolkit with:

- A Rust core library and CLI
- A WASM API package with TypeScript tests
- A Python extension module via PyO3 with pytest coverage

## Repository layout

- `src/` - core Rust wrapper library and CLI
- `wasm-api/` - WASM bindings and TypeScript tests
- `python-api/` - PyO3 Python extension and pytest tests
- `relaxng-rust/` - upstream relaxng crates (submodule)

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
