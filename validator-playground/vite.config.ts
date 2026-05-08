import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import wasm from "vite-plugin-wasm";

// https://vite.dev/config/
export default defineConfig({
    // Use relative asset paths so the app can be served from a subpath.
    base: "./",
    plugins: [react(), wasm()],
    worker: {
        format: "es",
        plugins: () => [wasm()],
    },
});
