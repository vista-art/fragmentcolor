// Step 1 — load, set, render (JS port).
//
// The JS constructor cannot perform network requests (WASM constructors
// can't be async), so URL-based shaders go through Shader.fetch. Every
// other line matches the Rust example one-for-one.

import { Shader } from "fragmentcolor";

// #region: setup
export async function setup(_renderer, _target) {
    const shader = await Shader.fetch("https://fragmentcolor.org/triangle.wgsl");
    shader.set("color", [0.95, 0.30, 0.42, 1.0]);
    return { shader };
}
// #endregion: setup

// #region: frame
export function frame(state, renderer, target, time, _size) {
    const r = 0.5 + 0.45 * Math.sin(time * 0.7);
    const g = 0.5 + 0.45 * Math.cos(time * 0.5 + 1.7);
    const b = 0.5 + 0.45 * Math.sin(time * 0.9 + 3.1);
    state.shader.set("color", [r, g, b, 1.0]);
    renderer.render(state.shader, target);
}
// #endregion: frame
