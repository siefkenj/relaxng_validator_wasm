import { defineConfig } from "vitest/config";
import wasm from "vite-plugin-wasm";

export default defineConfig({
  plugins: [wasm()],
  test: {
    include: ["wasm-api/tests/**/*.test.ts"],
    pool: "forks",
    testTimeout: 15000,
  },
});
