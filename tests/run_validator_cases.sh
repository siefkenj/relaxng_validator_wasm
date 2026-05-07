#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

BIN_NAME="relaxng-validator-wasm"

run_expect_ok() {
  local label="$1"
  shift

  echo "[OK-CASE] $label"
  local output
  if ! output="$(cargo run --quiet --bin "$BIN_NAME" -- "$@" 2>&1)"; then
    echo "Command failed unexpectedly for case: $label"
    echo "$output"
    exit 1
  fi

  local compact
  compact="$(printf '%s' "$output" | tr -d '[:space:]')"
  if ! printf '%s' "$compact" | grep -Fq '"errors":[]'; then
    echo "Expected no validation errors for case: $label"
    echo "$output"
    exit 1
  fi
}

run_expect_err() {
  local label="$1"
  shift

  echo "[ERR-CASE] $label"
  local output
  set +e
  output="$(cargo run --quiet --bin "$BIN_NAME" -- "$@" 2>&1)"
  local status=$?
  set -e

  if [ "$status" -eq 0 ]; then
    echo "Command succeeded unexpectedly for case: $label"
    echo "$output"
    exit 1
  fi

  local compact
  compact="$(printf '%s' "$output" | tr -d '[:space:]')"
  if ! printf '%s' "$compact" | grep -Fq '"errors":[' || printf '%s' "$compact" | grep -Fq '"errors":[]'; then
    echo "Expected one or more validation errors for case: $label"
    echo "$output"
    exit 1
  fi
}

run_expect_ok "pretext.rnc + test-good.xml" tests/pretext.rnc tests/test-good.xml
run_expect_ok "pretext.rng + test-good.xml" tests/pretext.rng tests/test-good.xml
run_expect_ok "pretext-dev.rnc pretext.rnc + test-good.xml" tests/pretext-dev.rnc tests/pretext.rnc tests/test-good.xml

run_expect_err "pretext.rnc + test-bad.xml" tests/pretext.rnc tests/test-bad.xml

echo "All validator CLI checks passed."
