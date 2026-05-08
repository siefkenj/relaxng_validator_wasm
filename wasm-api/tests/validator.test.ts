import { describe, it, expect, beforeAll } from "vitest";
import type {
    WasmValidator,
    ValidationResult,
    WasmValidationError,
} from "../pkg/relaxng_validator_wasm_api.js";
import * as relaxngValidator from "../pkg/relaxng_validator_wasm_api.js";

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

const MANY_CHOICES_SCHEMA = JSON.stringify({
    "main.rnc":
        "start = element root { element a { text } | element b { text } | element c { text } | element d { text } | element e { text } | element f { text } | element g { text } | element h { text } }",
});

const MANY_ATTRS_SCHEMA = JSON.stringify({
    "main.rnc":
        "start = element book { attribute a { text }?, attribute b { text }?, attribute c { text }?, attribute d { text }?, attribute e { text }?, attribute f { text }?, attribute g { text }?, attribute h { text }?, element title { text } }",
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
        const result = relaxngValidator.validate(
            SIMPLE_SCHEMA,
            '<?xml version="1.0"?><root>hello</root>',
        );
        expect(result.errors).toHaveLength(0);
    });

    it("returns errors for an invalid document", () => {
        const result = relaxngValidator.validate(
            SIMPLE_SCHEMA,
            '<?xml version="1.0"?><root><unexpected/></root>',
        );
        expect(result.errors.length).toBeGreaterThan(0);
        expect(notAllowedErrors(result)[0].token).toMatchObject({
            type: "ElementStart",
            span: { text: "unexpected", start: 28, end: 38 },
        });
    });

    it("lists expected elements on a wrong-element error", () => {
        const result = relaxngValidator.validate(
            CHOICE_SCHEMA,
            '<?xml version="1.0"?><root><baz/></root>',
        );
        const errs = notAllowedErrors(result);
        expect(errs.length).toBeGreaterThan(0);

        expect(errs[0]).toMatchObject({
            token: {
                type: "ElementStart",
                span: { text: "baz", start: 28, end: 31 },
            },
            expected_elements: expect.arrayContaining(["foo", "bar"]),
            expected_attributes: [],
        });
    });

    it("returns stable, deduplicated expected elements for long choice lists", () => {
        const xml = '<?xml version="1.0"?><root><zzz/></root>';
        const snapshots: string[][] = [];

        for (let i = 0; i < 10; i++) {
            const result = relaxngValidator.validate(MANY_CHOICES_SCHEMA, xml);
            const err = notAllowedErrors(result).find(
                (e) => e.token.type === "ElementStart",
            );
            expect(err).toBeDefined();
            snapshots.push(err!.expected_elements);
        }

        const baseline = snapshots[0];
        for (const s of snapshots) {
            expect(s).toEqual(baseline);
            expect(new Set(s).size).toBe(s.length);
        }

        expect(baseline).toEqual(["a", "b", "c", "d", "e", "f", "g", "h"]);
    });

    it("token on a wrong-element error is a structured object with populated fields", () => {
        // Regression: serde_json::Value::Object serialized through serde_wasm_bindgen
        // v0.4 produces a JS Map, which JSON.stringify renders as "{}".
        const result = relaxngValidator.validate(
            CHOICE_SCHEMA,
            '<?xml version="1.0"?><root><baz/></root>',
        );
        const errs = notAllowedErrors(result);
        expect(errs.length).toBeGreaterThan(0);

        expect(errs[0].token).toEqual({
            type: "ElementStart",
            prefix: { text: "", start: 0, end: 0 },
            local: { text: "baz", start: 28, end: 31 },
            span: { text: "baz", start: 28, end: 31 },
        });
    });

    it("reports expected_elements empty when content model is text", () => {
        const result = relaxngValidator.validate(
            SIMPLE_SCHEMA,
            '<?xml version="1.0"?><root><unexpected/></root>',
        );
        const errs = notAllowedErrors(result);
        expect(errs.length).toBeGreaterThan(0);
        expect(errs[0]).toMatchObject({
            token: {
                type: "ElementStart",
                span: { text: "unexpected", start: 28, end: 38 },
            },
            expected_elements: [],
        });
    });

    it("lists expected attributes on a bad-attribute error", () => {
        const result = relaxngValidator.validate(
            ATTRS_SCHEMA,
            '<?xml version="1.0"?><book bad-attr="x"><title>Hi</title></book>',
        );
        const attrErr = notAllowedErrors(result).find(
            (e) => e.expected_attributes.length > 0,
        );
        expect(attrErr).toBeDefined();
        expect(attrErr!).toMatchObject({
            token: {
                type: "Attribute",
                span: { text: 'bad-attr="x"', start: 27, end: 39 },
            },
            expected_attributes: expect.arrayContaining(["isbn"]),
        });
    });

    it("returns stable, deduplicated expected attributes across repeated validations", () => {
        const xml =
            '<?xml version="1.0"?><book bad="x"><title>Hi</title></book>';
        const snapshots: string[][] = [];

        for (let i = 0; i < 10; i++) {
            const result = relaxngValidator.validate(MANY_ATTRS_SCHEMA, xml);
            const err = notAllowedErrors(result).find(
                (e) =>
                    e.token.type === "Attribute" &&
                    e.expected_attributes.length > 0,
            );
            expect(err).toBeDefined();
            snapshots.push(err!.expected_attributes);
        }

        const baseline = snapshots[0];
        for (const s of snapshots) {
            expect(s).toEqual(baseline);
            expect(new Set(s).size).toBe(s.length);
        }

        expect(baseline).toEqual(["a", "b", "c", "d", "e", "f", "g", "h"]);
    });

    it("reports expected_attributes empty for element-level errors", () => {
        const result = relaxngValidator.validate(
            CHOICE_SCHEMA,
            '<?xml version="1.0"?><root><baz/></root>',
        );
        const errs = notAllowedErrors(result);
        expect(errs.length).toBeGreaterThan(0);
        for (const err of errs) {
            expect(err).toMatchObject({ expected_attributes: [] });
        }
    });

    it("each error has a type discriminant", () => {
        const result = relaxngValidator.validate(
            SIMPLE_SCHEMA,
            '<?xml version="1.0"?><root><bad/></root>',
        );
        expect(result.errors).toEqual([
            {
                type: "NotAllowed",
                token: {
                    type: "ElementStart",
                    prefix: { text: "", start: 0, end: 0 },
                    local: { text: "bad", start: 28, end: 31 },
                    span: { text: "bad", start: 28, end: 31 },
                },
                expected_elements: [],
                expected_attributes: [],
            },
        ]);
    });
});

// ---------------------------------------------------------------------------
// compile_validator() + WasmValidator.validate()
// ---------------------------------------------------------------------------

describe("compile_validator() / WasmValidator", () => {
    let validator: WasmValidator;

    beforeAll(() => {
        validator = relaxngValidator.compile_validator(SIMPLE_SCHEMA);
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
        const v = relaxngValidator.compile_validator(schema);
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
        const result = relaxngValidator.validate(
            vfsJson,
            '<?xml version="1.0"?><root>hello</root>',
        );
        expect(result.errors).toHaveLength(0);
    });
});
