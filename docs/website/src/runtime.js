// One WASM boot per page, shared by every component that draws with
// FragmentColor. wasm-bindgen's init only skips work once a boot has
// finished, so two components booting at once would each instantiate the
// module, and objects from the first would point into the second's memory.
// Registry slugs resolve against the site's own /shaders/ tree, so dev works
// offline.

let ready;

export function loadFragmentColor() {
  ready ??= (async () => {
    const fc = await import("fragmentcolor");
    const wasmUrl = (await import("fragmentcolor/fragmentcolor_bg.wasm?url")).default;
    await fc.default({ module_or_path: wasmUrl });
    fc.Shader.setRegistry("/shaders/");
    return fc;
  })().catch((err) => {
    ready = undefined;
    throw err;
  });
  return ready;
}
