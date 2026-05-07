# python-api

PyO3-based Python extension for the RELAX NG validator.

## Prerequisites

- Rust toolchain
- Python 3.9+
- `maturin`

## Set up a virtual environment

From the repository root:

```bash
python3 -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
pip install pytest maturin
```

## Build and install for development

From the repository root (with virtual environment activated):

```bash
maturin develop
```

This compiles the extension and installs it into the active virtual environment in editable mode.

## Run Python tests

From the repository root (with virtual environment activated):

```bash
pytest
```

Or run only the Python API tests:

```bash
pytest python-api/tests -v
```

## Create a wheel

From the repository root (with virtual environment activated):

```bash
maturin build --release
```

The wheel will be created under:

- `target/wheels/`

Install the generated wheel manually (example):

```bash
pip install target/wheels/relaxng_validator-0.1.0-*.whl
```

## Notes

- The Python module name is `relaxng_validator`.
- Build configuration is in the repository root `pyproject.toml` and points to `python-api/Cargo.toml`.
