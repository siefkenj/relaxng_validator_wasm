import { expose } from "comlink";
import { validate } from "relaxng-validator-wasm-api";
import type { ValidatorWorkerApi } from "./validatorWorkerApi.ts";

const workerApi: ValidatorWorkerApi = {
    validate(vfsJson, xmlText) {
        return validate(vfsJson, xmlText);
    },
};

expose(workerApi);
