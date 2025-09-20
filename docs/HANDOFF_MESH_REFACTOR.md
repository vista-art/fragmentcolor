# Mesh Refactor Handoff

This is a copy‑paste prompt for the next agent to continue the full Mesh refactor.

Context
- Project: FragmentColor (Rust + wgpu). Public docs live in docs/api and feed the website.
- Build.rs enforces docs coverage via #[lsp_doc] and exports the website content.
- Goal: Make vertex attribute mapping fully reflection‑driven from the Shader; remove all implicit assumptions (e.g., location(0) special cases, uv/color conveniences).

What we achieved in this session
1) Vertex foundations
   - VertexPosition is now a pure newtype over Vec4 (xyzw, w defaults to 1.0).
   - Vertex infers its “dimensions” (2D or 3D) from constructor input and uses that to select the vertex format for the position attribute (Float32x2 or Float32x3).
   - Vertex properties are added with `Vertex::with(key, value)`. On first use of a key, the vertex assigns the next available @location(N) to that property; subsequent uses reuse the same location. Per‑vertex property locations are stored in `prop_locations: HashMap<String, u32>`.
   - Instances no longer copy position implicitly; they clone the vertex’s properties and property‑location map.

2) Renderer mapping
   - Removed the “location(0) is position” rule. Mapping is now:
     - instance explicit location (if first instance specifies an index for a key) → bind instance stream
     - vertex explicit location (position or a property, if first vertex specifies an index) → bind vertex stream
     - fallback by name (instance first, then vertex)
   - Type mismatches between shader input format and mesh property format return a descriptive error.

3) Mesh schema/packing
   - Vertex schema includes a single `position` key with a 2‑ or 3‑component format.
   - All other properties are included by sorted name order and packed by type.

4) Docs updated
   - docs/api/core/mesh/mesh.md and website MDX updated to reflect reflection‑driven mapping and removal of special‑cases.
   - docs/api/core/vertex/with.md updated with location assignment behavior and planned explicit control.

What’s left to do (original checklist adapted and extended)
1) Remove uv/color convenience APIs entirely (already removed from docs/examples; code still had With UV/Color earlier but has been refactored to `with`). Ensure no lingering references in platforms/ examples.
2) Provide explicit control over property locations
   - API design: chained fluent API `vertex.set(key, value).at(index)` to pin a property to a specific @location(N). Under the hood:
     - `set` stores the value and, if the key is new, assigns the next location; returns a builder with `.at(index)` to override.
     - `.at(index)` updates `prop_locations[key] = index`.
   - Keep the existing `with(key, value)` as sugar for `set(key, value)`.
   - Optional: Add `vertex.position_at(index)` if you want to let position move off 0. If omitted, keep 0 by default.

3) Shader → Vertex construction helpers
   - Prefer `Vertex::from_shader(&Shader)` constructor that reflects the shader’s vertex inputs and pre‑builds a skeleton Vertex layout:
     - Inspects vertex inputs (name, location, format) via naga reflection.
     - Seeds `prop_locations` to match the reflected locations for known property names.
     - Does not supply values; just establishes the layout. Provide fluent setters afterward.
   - Alternative (if needed): `Shader::get_vertex_layout()` returning `Vec<{ name, location, format }>`.

4) Renderer: simplify priority logic
   - Once the explicit `.at(index)` API is done and `Vertex::from_shader()` is available, we can simplify mapping to:
     - Prefer explicit property index → map by location to matching stream (instance vs vertex) using `prop_locations`.
     - Otherwise, name fallback.
   - Document the precedence clearly.

5) Tests and examples
   - Update tests to exercise:
     - Name‑only mapping
     - Explicit location mapping via `.set(...).at(n)`
     - Conflicting names in vertex vs instance (instance precedence)
     - Position at a non‑zero location if `position_at()` is implemented
   - Update rust/js/python examples to avoid any legacy helpers and to demonstrate `.with` and `.set(...).at(n)` flows.

6) Docs
   - Add docs for the new fluent API (`vertex.set` and `.at`) and for `Vertex::from_shader`.
   - Clarify in Mesh/Vertex docs: there is no special case for location(0); mapping is driven by shader reflection and optional explicit indices.

Design guidance and constraints
- Naming and style (from project rules):
  - Avoid abbreviations; prefer clear names: “dimensions”, “vector”, “property_locations” (internally you may keep `prop_locations`, but public API should be readable).
  - Public methods should be thin delegators, with logic in internal modules.
  - Avoid unwrap/expect/panic in library code; use thiserror for errors.
  - Prefer parking_lot over std mutexes.
- Docs validation: Every public item with #[lsp_doc] must have a corresponding doc. Adding docs for planned APIs is okay, but ensure doc pages include a “## Example” section to pass validators.

Implementation sketch for the explicit location API
- Public API (Rust):
  - `impl Vertex { fn set<V: Into<VertexValue>>(&mut self, key: &str, value: V) -> PropertyBuilder; }`
  - `struct PropertyBuilder<'a> { vertex: &'a mut Vertex, key: String }`
  - `impl<'a> PropertyBuilder<'a> { fn at(self, index: u32) -> &'a mut Vertex { /* update vertex.prop_locations[key] = index */ } }`
  - `with(key, value)` stays as a convenience: constructs, sets default auto index, returns `self`.
  - Optionally, `fn position_at(&mut self, index: u32)` to pin position; default remains 0.

- Vertex::from_shader(&Shader):
  - Use existing reflection (we already have `reflect_vertex_inputs()` internally) to get a list of `{ name, location, format }`.
  - Seed `prop_locations` with these locations for all names that are not “position”. For position, either pin to 0 or to the location you want, based on convention. If you support `position_at`, read the shader input name that corresponds to position (typically a vec2/vec3 named pos/position) and assign accordingly.
  - Return a Vertex with empty property values and the preconfigured location map.

Current code state summary
- Vertex: pure Vec4 position via builtins::VertexPosition; dimensions on Vertex; with(key, value) auto‑indexes; properties + prop_locations tracked.
- Mesh: derives `position` (2/3 comps) and packs properties by name/type; instances do not copy position.
- Renderer: reflection‑driven mapping with priority (instance/vertex explicit index, then name); no special location(0) rule.
- Docs: Mesh and Vertex pages updated to reflection‑driven mapping and planned `.set(...).at(n)` and `Vertex::from_shader()`.

Acceptance checks after completion
- cargo test / cargo clippy with -D warnings passes.
- build.rs doc validation passes (no missing docs for public items).
- Examples compile/run with updated APIs; CI healthchecks still pass.
