## My clarifying answers

1) Canonical docs domain:
•  fragmentcolor.org (please replace all .com instances, they were probably a type)

2) Exact scope of “top-level public API” for Part 1 and the Reflection/Validation build step:
•  Please confirm the set we should consider “top-level public API structs” for docs and validation. Proposed:
◦  Shader
◦  Renderer
◦  Pass
◦  Frame
◦  Target (trait/abstraction)
◦  TextureTarget (the headless/offscreen Target) - those are OK. Target is a WIP, I'm thinking on unifying both WindowTarget and TextureTarget into an object that can handle both

•  Should we exclude: RenderContext, Color, Region, Size, Sampler, internal platform adapters, and other helper types from the public docs and validation?
- Yes, they are either internal or in draft stage

3) Method file naming and coverage:
•  Confirm file naming convention: docs/api/{object}/{method_snake_case}.md (using Rust method names, not per-language wrapper names).
- Rust snake_case convention for file names. However, we still need to adapt the examples for each language to use their native versions.

•  For multi-language method names (create_target in Rust wrappers vs createTarget in JS), we’ll document once at docs/api/renderer/create_target.md and point all wrappers’ lsp_doc to that same file. OK?
- That's OK, that's exactly what I expect. We could create all examples from all languages directly in the md file, that will make things easier. Then we validate at build time all of them are present. Don't let the program compile if the documentation of the public API is not properly written first.

4) Validation rules details:
•  The current spec says each file must “exist, be non-empty, contain ##example, contain top-level #{struct_name}, and contain at least one phrase below it.”
•  Do these checks apply to:
◦  The object page (docs/api/{object}/{object}.md) only?
◦  Or to every method file too?
- I think rules are for both. In the MD file, function names will be h1, but converted to h3 in the website's mdx.

•  Case sensitivity: Should “## Example” be accepted, or must it be “## example” exactly?
- “## Example” with uppercase first letter.

•  For method files, do you want a required “## Example” section as well, or just a heading and descriptive text?
- For methods it is even more important.

5) Method ordering in website output:
•  “Same order as source code” — confirm that ordering should be taken from the core Rust impl blocks (src/{object}/mod.rs), not from per-platform bindings.
- Yes. If the bindings have different order, tell me and I'll manually reorder them (I think they're mostly consistent with the Rust counterpart).

•  JS-only or Python-only methods (e.g., Shader::fetch for JS): should they be included in the website Methods list for that object? If yes, where should they appear relative to the core Rust methods?
- Exclusive methods should have their own files, so the second step with grab them. We can write (for JavaScript Only) in the title.

6) Example translation (Rust → Python/JS) in the final MDX:
•  I will implement a heuristic that:
◦  Maps Rust “use fragmentcolor::X; let x = X::new(...);” to:
◦  Python: from fragmentcolor import X; x = X(...)
◦  JS: import { X } from "fragmentcolor"; const x = new X(...)
◦  Translates renderer.create_texture_target([...]) to:
◦  Python: renderer.create_texture_target([...])
◦  JS: await renderer.createTextureTarget([...])
◦  Removes borrowing (&) and Result handling, converting to the straightforward usage patterns shown in your examples.
•  Please confirm these translation rules (and add any specific gotchas like naming differences or async requirements you want enforced).

- How about we pick the Python/Js examples from their real bindings healthcheck folder? We could enforce a rule that every public method and struct require a healthcheck live example. We can create a simple wrapper with known comment annotations to facilitate copying from the script. What do you think? We could enforce a compile time rule that examples should be present for all platforms. This way we can test them in CI and make everything more reliable before launch.

7) lsp_doc replacement scope:
•  Replace doc-comments with #[lsp_doc(...)] on public objects/methods only, correct? Internal/private items can keep local /// comments.
- Yes

•  For the shared methods that already have lsp_doc in web/python wrappers, should I also add lsp_doc to the corresponding core Rust methods (e.g., Renderer::create_texture_target) so rustdoc hover is consistent in Rust too?
- Yes. If any code is used by the wrappers, it should be public in Rust if it's not already. Maybe there are exceptions I'm not aware of.

8) Commit details:
•  Any preferred commit message format? I propose:
◦  chore(docs): complete API docs; add lsp_doc; add build.rs validation + website sync for v0.10.7
•  Any constraints about splitting into multiple commits (Part 1 vs Part 2), or squash into one?
- Don't worry about commits. I'll take care of them. I want to do everything in one batch.

I saved these answers in prompt in CLARIFYING_QUESTIONS.md#

## Agent's Plan

I will now create a structured execution plan based on your confirmations, including exact file-level changes, validations, and build-time automation details. This helps ensure nothing is missed and gives us a clear checklist to follow.
Normalize and complete docs/api
Wire lsp_doc for public API and remove in-source doc comments
Establish example sources for JS/Python (healthchecks)
Refactor build.rs (Reflection, Validation, Website Update)
QA and final checks
Great — here’s the finalized execution blueprint for v0.10.7 docs completion and the build-time automation. I’ve incorporated your confirmations and mapped them into concrete steps, with exact file-level targets and validations.

Summary of decisions
•  Canonical domain: fragmentcolor.org (replace all .com instances in docs)
•  Public API scope (document + validate): Shader, Renderer, Pass, Frame, Target, TextureTarget
•  Exclude from docs/validation: RenderContext, Color, Region, Size, Sampler, platform adapters, helpers
•  Method docs naming: docs/api/{object}/{method_snake_case}.md (Rust names), but include multi-language examples in the same md file
•  Validation enforceable in build.rs:
•  Apply to both object files and method files
•  Require H1 and “## Example” (capital E) sections
•  Require JS and Python examples sourced from real healthchecks (via annotated blocks) in addition to the Rust snippet in the .md
•  If anything is missing → generate stubs, continue iterating, and fail at the end with a detailed error list
•  Ordering: keep method order from core Rust impl blocks; include platform-only methods with appropriate title markers “(JavaScript Only)” etc.
•  lsp_doc: replace doc-comments on public items with #[lsp_doc("docs/api/...")], including the core Rust method that wrappers use (e.g., create_texture_target).

Part 1 — Ensure documentation is complete

What to change
•  Standardize links and clean structure
•  Search-and-replace in docs/api: fragmentcolor.com → fragmentcolor.org
•  Remove .mdx from docs/api; source-of-truth there is .md only
•  Create docs/api/target/target.md (convert docs/api/target/target.mdx)
•  Fill missing docs
•  Frame: add docs/api/frame/constructor.md and docs/api/frame/add_pass.md
•  Pass: add docs/api/pass/set_viewport.md and docs/api/pass/set_clear_color.md
•  Consider also documenting public Pass methods (compute.md, from_shader.md, get_input.md, load_previous.md), since they are public and you want full coverage
•  TextureTarget: create docs/api/texture_target/texture_target.md (at minimum a description and examples; you can keep methods aligned with Target trait, i.e., size/resize)
•  Replace public rustdoc comments with lsp_doc on:
•  src/shader/mod.rs
◦  Add lsp_doc to the struct: #[lsp_doc("docs/api/shader/shader.md")]
◦  Methods already wired
•  src/renderer/mod.rs
◦  Add lsp_doc on create_texture_target(): #[lsp_doc("docs/api/renderer/create_texture_target.md")]
•  src/pass/mod.rs
◦  Add lsp_doc for add_shader (docs exist); set_viewport and set_clear_color (new docs you’ll add); consider compute/from_shader/get_input/load_previous as well
•  src/frame/mod.rs
◦  Add lsp_doc to struct: #[lsp_doc("docs/api/frame/frame.md")]
◦  Add lsp_doc to new() and add_pass()
•  Target trait
◦  Use docs/api/target/target.md as the source; if lsp_doc supports trait items, annotate; otherwise leave as-is (no existing doc comments here)
•  Ensure every docs/api/{object}/{object}.md and every docs/api/{object}/{method}.md uses:
•  H1 line: “# ObjectName” or “# method_signature”
•  Has a “## Example” section with at least:
◦  One Rust code block (doctestable)
◦  The build will later inject JS + Python from healthchecks into the website output

What to check (commands to run yourself)
•  cargo fmt --all --check
•  cargo clippy --all-targets --all-features -- -D warnings
•  cargo test (to compile/run Rust doctests inside docs injected via lsp_doc)

Relevant context
•  Present docs/api files:
•  shader: shader.md, constructor.md, set.md, get.md, list_uniforms.md, list_keys.md, fetch.md
•  renderer: renderer.md, constructor.md, render.md, create_target.md, create_texture_target.md
•  pass: pass.md, constructor.md, add_shader.md
•  frame: frame.md
•  target: target.mdx (convert to .md)
•  Source files to edit:
•  src/shader/mod.rs
•  src/renderer/mod.rs
•  src/pass/mod.rs
•  src/frame/mod.rs
•  src/target/mod.rs (for trait docs source, no needed lsp_doc if not supported)
•  Older website content (to be overwritten by generator):
•  docs/website/src/content/docs/api/{shader.mdx, renderer.mdx, pass.mdx, frame.mdx, target.mdx}

Part 2 — Build-time scripts in build.rs

Pass 1: Reflection (update existing code)
•  Filter to just top-level public API objects: Shader, Renderer, Pass, Frame, Target, TextureTarget
•  Extract public methods in file order from core Rust impls (not platform wrappers) and preserve this order
•  Include exclusive platform methods (like Shader::fetch for JS) by allow-listing names and marking them as “JS-only” in metadata for display purposes
•  Output a phf map (or a static structure) that preserves order for each object

Pass 2: Validation (strict, but non-aborting until end)
•  For each object and method from the reflection pass:
•  File exists at docs/api/{object}/{object or method}.md
•  File is not empty
•  Contains:
◦  H1: “# {ObjectName}” for object pages; “# {method_signature_or_name}” for method pages
◦  “## Example” section (exact “Example” casing), with at least one line after it
•  All links within the file use fragmentcolor.org (flag any mismatch)
•  Healthchecks coverage:
•  Parse platforms/python/healthcheck.py and platforms/web/healthcheck/main.js to find annotated DOC blocks per object/method
•  Proposed marker format:
◦  Python: # DOC: Renderer.create_texture_target (begin) ... # DOC: (end)
◦  JavaScript: // DOC: Renderer.create_texture_target (begin) ... // DOC: (end)
•  Require that each object/method reflected appears in both Python and JS healthchecks; if missing, record as invalid
•  Auto-create missing doc files:
•  If file/dir missing, create skeleton with H1, one-sentence description placeholder, and a “## Example” section containing a TODO Rust code fence
•  Mark dirty=true for any new or invalid files
•  End behavior:
•  If dirty == true, print a grouped list of all problems to stderr (missing files, empty files, missing sections, missing healthcheck examples, wrong domains) and then fail the build

Pass 3: Website update (only if validation passed)
•  Generate docs/website/src/content/docs/api/{object}.mdx by:
•  Writing frontmatter:
◦  title: Capitalized object name
◦  description: first paragraph (or first non-empty text block) from {object}.md
•  Insert “## Description” with the full content of {object}.md
•  Insert “## Methods” and for each method (in reflection order):
◦  Append its .md content
◦  Downshift headings within method docs by one level so method headings become “###”
•  For “## Example(s)”:
◦  Keep Rust snippet from .md
◦  Insert a “### Javascript” block from healthcheck JS DOC for that object/method
◦  Insert a “### Python” block from healthcheck Python DOC
◦  If a platform-only method, include it with a label in the method title, e.g., “- ### fetch(url) (JavaScript Only)”
•  Overwrite existing website API files fully

Additional constraints and rules to apply
•  Use parking_lot alternatives already present (RwLock is in use; keep it)
•  Avoid unwrap()/expect()/panic() in library code (build.rs may panic at end if dirty; that’s acceptable)
•  Keep names simple and clear; method docs H1 should be method signature or name
•  Prefer named lifetimes if any new public types are introduced (unlikely here)
•  Don’t change internal draft/helper APIs’ visibility

Optional small cleanups
•  docs/website/package.json has "fragmentcolor": "0.10.6" — you might bump to 0.10.7 with this release
•  Leave commits to be squashed as you prefer

Key repo references I used
•  docs/api contents present and gaps:
•  Present: shader/, renderer/, pass/*, frame/frame.md, target/target.mdx
•  Missing: frame/constructor.md, frame/add_pass.md, pass/set_viewport.md, pass/set_clear_color.md, texture_target/*
•  Source files to update with lsp_doc:
•  src/shader/mod.rs (struct)
•  src/renderer/mod.rs (create_texture_target)
•  src/pass/mod.rs (add_shader, set_viewport, set_clear_color, and other public methods)
•  src/frame/mod.rs (struct, new(), add_pass())
•  Healthchecks to harvest examples:
•  platforms/python/healthcheck.py
•  platforms/web/healthcheck/main.js
•  build.rs current state:
•  Reflection already implemented to enumerate items; needs filtering, ordering, validation, and website generation phases added

What I’ll do next
•  I created a to-do plan with 5 grouped tasks to execute in order:
  1) Normalize and complete docs/api
  2) Wire lsp_doc and remove in-source public doc comments
  3) Establish DOC anchors for examples in healthchecks
  4) Refactor build.rs to add Reflection, Validation (with auto-stub), and Website Update
  5) Run full QA (fmt, clippy -D warnings, tests, website build)

If you want me to start applying changes now, say “Proceed,” and I’ll begin with Part 1 (docs/api normalization and missing files) and wire lsp_doc where needed, then move on to the build script work.
