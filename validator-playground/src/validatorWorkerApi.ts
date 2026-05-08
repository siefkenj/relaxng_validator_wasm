import type { WasmValidationError } from "relaxng-validator-wasm-api";

export type ValidationResponse = {
    errors: WasmValidationError[];
};

export type ValidatorWorkerApi = {
    validate(vfsJson: string, xmlText: string): ValidationResponse;
};
