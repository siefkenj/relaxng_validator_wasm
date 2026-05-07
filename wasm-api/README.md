# wasm-api

Rust + WASM API for the RELAX NG validator.

## Prerequisites

- Rust toolchain (with `wasm32-unknown-unknown` target)
- `wasm-pack`
- Node.js and npm

## Build

From the repository root:

```bash
npm run build
```

This runs:

```bash
wasm-pack build wasm-api --target bundler
```

If you specifically want a node-usable package output in `wasm-api/pkg`:

```bash
npm run build:node
```

## Test

From the repository root:

```bash
npm test
```

What this does:

1. Rebuilds the WASM package for tests.
2. Runs Vitest against `wasm-api/tests/validator.test.ts`.

## Notes

- The generated package is written to `wasm-api/pkg`.
- TypeScript test and Vitest configuration live at the repository root (`tsconfig.json`, `vitest.config.ts`, `package.json`).
