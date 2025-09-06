import { defineConfig } from "vite";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

// Vite config enabling WebAssembly ESM integration and top-level await.
// The worker.plugins block mirrors the main plugins to support WASM/TLA inside Web Workers if used.
export default defineConfig({
  plugins: [wasm(), topLevelAwait()],
  worker: {
    // Vite 7: worker.plugins must be a function returning an array.
    plugins: () => [wasm(), topLevelAwait()],
  },
});

