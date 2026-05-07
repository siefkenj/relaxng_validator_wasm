import { describe, it, expect, beforeAll } from "vitest";
import type {
    WasmValidator,
    ValidationResult,
    WasmValidationError,
} from "../pkg/relaxng_validator_wasm_api.js";
import * as wasm from "../pkg/relaxng_validator_wasm_api.js";

// pkg is built by `wasm-pack build wasm-api --target bundler --out-dir pkg`.
// vite-plugin-wasm handles the WASM binary loading transparently.

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const SIMPLE_SCHEMA = JSON.stringify({
    "main.rnc": "start = element root { text }",
});

const CHOICE_SCHEMA = JSON.stringify({
    "main.rnc":
        "start = element root { element foo { text } | element bar { text } }",
});

const ATTRS_SCHEMA = JSON.stringify({
    "main.rnc":
        "start = element book { attribute isbn { text }, attribute year { text }?, element title { text } }",
});

function notAllowedErrors(result: ValidationResult) {
    return result.errors.filter(
        (e): e is Extract<WasmValidationError, { type: "NotAllowed" }> =>
            e.type === "NotAllowed",
    );
}

// ---------------------------------------------------------------------------
// validate() — one-shot function
// ---------------------------------------------------------------------------

describe("validate()", () => {
    it("returns empty errors for a valid document", () => {
        const result = wasm.validate(
            SIMPLE_SCHEMA,
            '<?xml version="1.0"?><root>hello</root>',
        );
        expect(result.errors).toHaveLength(0);
    });

    it("returns errors for an invalid document", () => {
        const result = wasm.validate(
            SIMPLE_SCHEMA,
            '<?xml version="1.0"?><root><unexpected/></root>',
        );
        expect(result.errors.length).toBeGreaterThan(0);
    });

    it("lists expected elements on a wrong-element error", () => {
        const result = wasm.validate(
            CHOICE_SCHEMA,
            '<?xml version="1.0"?><root><baz/></root>',
        );
        const errs = notAllowedErrors(result);
        expect(errs.length).toBeGreaterThan(0);

        const first = errs[0];
        expect(first.expected_elements).toContain("foo");
        expect(first.expected_elements).toContain("bar");
    });

    it("reports expected_elements empty when content model is text", () => {
        const result = wasm.validate(
            SIMPLE_SCHEMA,
            '<?xml version="1.0"?><root><unexpected/></root>',
        );
        const errs = notAllowedErrors(result);
        expect(errs.length).toBeGreaterThan(0);
        expect(errs[0].expected_elements).toHaveLength(0);
    });

    it("lists expected attributes on a bad-attribute error", () => {
        const result = wasm.validate(
            ATTRS_SCHEMA,
            '<?xml version="1.0"?><book bad-attr="x"><title>Hi</title></book>',
        );
        // Find a NotAllowed error that lists expected attributes (the bad-attr error)
        const attrErr = notAllowedErrors(result).find(
            (e) => e.expected_attributes.length > 0,
        );
        expect(attrErr).toBeDefined();
        expect(attrErr!.expected_attributes).toContain("isbn");
    });

    it("reports expected_attributes empty for element-level errors", () => {
        const result = wasm.validate(
            CHOICE_SCHEMA,
            '<?xml version="1.0"?><root><baz/></root>',
        );
        // All NotAllowed errors here are element-level (wrong child element)
        const errs = notAllowedErrors(result);
        expect(errs.length).toBeGreaterThan(0);
        for (const err of errs) {
            expect(err.expected_attributes).toHaveLength(0);
        }
    });

    it("each error has a type discriminant", () => {
        const result = wasm.validate(
            SIMPLE_SCHEMA,
            '<?xml version="1.0"?><root><bad/></root>',
        );
        for (const err of result.errors) {
            expect(typeof err.type).toBe("string");
        }
    });
});

// ---------------------------------------------------------------------------
// compile_validator() + WasmValidator.validate()
// ---------------------------------------------------------------------------

describe("compile_validator() / WasmValidator", () => {
    let validator: WasmValidator;

    beforeAll(() => {
        validator = wasm.compile_validator(SIMPLE_SCHEMA);
    });

    it("returns empty errors for a valid document", () => {
        const result = validator.validate(
            '<?xml version="1.0"?><root>hello</root>',
        );
        expect(result.errors).toHaveLength(0);
    });

    it("returns errors for an invalid document", () => {
        const result = validator.validate(
            '<?xml version="1.0"?><root><child/></root>',
        );
        expect(result.errors.length).toBeGreaterThan(0);
    });

    it("is reusable across multiple calls", () => {
        const good = '<?xml version="1.0"?><root>text</root>';
        const bad = '<?xml version="1.0"?><root><child/></root>';

        expect(validator.validate(good).errors).toHaveLength(0);
        expect(validator.validate(bad).errors.length).toBeGreaterThan(0);
        expect(validator.validate(good).errors).toHaveLength(0);
    });

    it("successive invalid documents each return errors", () => {
        const bad = '<?xml version="1.0"?><root><child/></root>';
        const r1 = validator.validate(bad);
        const r2 = validator.validate(bad);
        expect(r1.errors.length).toBeGreaterThan(0);
        expect(r2.errors.length).toBeGreaterThan(0);
    });
});

// ---------------------------------------------------------------------------
// First key as grammar entry point
// ---------------------------------------------------------------------------

describe("first key as grammar entry point", () => {
    it("uses the first key as the root grammar", () => {
        // Both keys are valid standalone grammars; only "main.rnc" (first) is used.
        const schema = JSON.stringify({
            "main.rnc": "start = element doc { text }",
            "other.rnc": "start = element other { text }",
        });
        const v = wasm.compile_validator(schema);
        expect(
            v.validate('<?xml version="1.0"?><doc>hi</doc>').errors,
        ).toHaveLength(0);
        expect(
            v.validate('<?xml version="1.0"?><other>hi</other>').errors.length,
        ).toBeGreaterThan(0);
    });
});

// ---------------------------------------------------------------------------
// VFS byte-array values
// ---------------------------------------------------------------------------

describe("VFS byte-array file content", () => {
    it("accepts schema content supplied as a byte array", () => {
        // "start = element root { text }" encoded as UTF-8 bytes
        const schema = "start = element root { text }";
        const bytes = Array.from(new TextEncoder().encode(schema));
        const vfsJson = JSON.stringify({ "main.rnc": bytes });
        const result = wasm.validate(
            vfsJson,
            '<?xml version="1.0"?><root>hello</root>',
        );
        expect(result.errors).toHaveLength(0);
    });
});
