mod swift {
    //! Swift transpilation — second phase.
    //!
    //! `convert::to_swift` runs the per-line emitter with `Lang::Swift`,
    //! producing a JS-shaped intermediate (Swift shares JS's control flow:
    //! `{...}` blocks, `;` terminators, `new Type(...)`, `const`, backtick
    //! template literals). This module finishes the job with the lexical
    //! swaps that don't fit a per-line model.
    //!
    //! Rules applied in order:
    //!
    //! 1. Backtick template literals (`...`) → triple-quoted strings (`"""..."""`)
    //!    so multi-line WGSL shaders translate directly.
    //! 2. Rewrite the fragmentcolor import (`import { X } from "fragmentcolor";` → `import FragmentColor`).
    //! 3. Strip trailing semicolons — Swift terminates statements at newlines.
    //! 4. Drop the `new` keyword (`new Renderer(...)` → `Renderer(...)`).
    //! 5. `const x = ...` → `let x = ...`.
    //! 6. `await foo(...)` → `try await foo(...)` (Swift requires `try` on async throws).
    //! 7. `null` → `nil`.
    //! 8. `document.createElement("canvas")` lines → dropped (iOS has no DOM).
    //!    The subsequent `createTarget(canvas)` → `createTextureTarget([800, 600])`.
    //! 9. `TextureRegion` → `TextureRegionMobile`, the mobile-only type that converts into it.
    //! 10. `x..y` range → `x..<y`, and `x..=y` → `x...y`.
    //! 11. `.Rgba` / `.Rgba8UnormSrgb` enum case first-letter lowercasing.
    //! 12. `.unwrap()` on a throwing init → `try!` prefix.
    //! 13. `Vertex.new([...])` → `Vertex([...])`
    //!     when an unlabeled init takes those arguments.
    //! 14. `try` / `await` before a statement whose calls throw or suspend.
    //!
    //! Rule 9 reads the `From` impls of the types uniffi exports for iOS.
    //! Rules 13 and 14 read the uniffi exports for iOS
    //! and the declarations in the hand-written `*+Extensions.swift` files.

    use std::collections::BTreeMap;
    use std::fs;

    use crate::reflect::{Build, CallKind, Reflection, StandIn, StrConst};
    use crate::source_text::{
        Nesting, arg_spans, closing, code_only, drop_semicolon_before_comment, extension_files,
        is_ident_char, read_ident, skip_space, split_args, string_end, top_level_words,
    };

    /// The extension files that add Swift overloads on top of uniffi's.
    const EXTENSIONS_DIR: &str = "platforms/swift/Sources/FragmentColor";

    /// Keywords that lead a statement no `try` or `await` can precede.
    const STATEMENT_KEYWORDS: &[&str] =
        &["return", "if", "guard", "for", "while", "switch", "else", "case", "defer", "throw"];

    /// What a call needs at its call site: `await`, `try`, or both.
    #[derive(Clone, Copy, Default)]
    struct Effects {
        is_async: bool,
        throws: bool,
    }

    impl Effects {
        fn merge(&mut self, other: Effects) {
            self.is_async |= other.is_async;
            self.throws |= other.throws;
        }

        fn keyword(self) -> Option<&'static str> {
            match (self.is_async, self.throws) {
                (true, true) => Some("try await"),
                (true, false) => Some("await"),
                (false, true) => Some("try"),
                (false, false) => None,
            }
        }
    }

    /// A Swift function or initializer, from uniffi or an extension file.
    struct Decl {
        owner: String,
        name: String,
        kind: CallKind,
        effects: Effects,
        /// Parameter types when every parameter is unlabeled.
        unlabeled: Option<Vec<String>>,
    }

    /// API facts the Swift rewrites read, derived from the source model.
    struct Facts {
        /// String constants the Swift bindings don't expose.
        consts: Vec<StrConst>,
        /// Crate types Swift names by the mobile-only type that converts into them.
        stand_ins: Vec<StandIn>,
        /// Instance methods by name, merged over every type that has one.
        /// A line rewriter can't type a receiver variable.
        methods: BTreeMap<String, Effects>,
        /// Static functions by type and name.
        statics: BTreeMap<(String, String), Effects>,
        /// Initializers by type, merged over overloads.
        inits: BTreeMap<String, Effects>,
        /// Parameter types of each unlabeled initializer, by type.
        unlabeled_inits: BTreeMap<String, Vec<Vec<String>>>,
    }

    static FACTS: std::sync::OnceLock<Facts> = std::sync::OnceLock::new();

    /// Derive the Swift rewrites' API facts. Runs once before any example converts.
    pub fn install(model: &Reflection) {
        let mut facts = Facts {
            consts: model.str_consts(Build::Ios),
            stand_ins: model.stand_ins(Build::Ios),
            methods: BTreeMap::new(),
            statics: BTreeMap::new(),
            inits: BTreeMap::new(),
            unlabeled_inits: BTreeMap::new(),
        };
        let exported = model.uniffi_fns(Build::Ios).into_iter().map(|f| Decl {
            effects: Effects {
                is_async: f.is_async,
                throws: f.throws,
            },
            // uniffi labels every parameter it generates.
            unlabeled: f.params.is_empty().then(Vec::new),
            owner: f.owner,
            name: f.name,
            kind: f.kind,
        });
        for decl in exported.chain(extension_decls()) {
            match decl.kind {
                CallKind::Method => facts.methods.entry(decl.name).or_default().merge(decl.effects),
                CallKind::Static => facts
                    .statics
                    .entry((decl.owner, decl.name))
                    .or_default()
                    .merge(decl.effects),
                CallKind::Init => {
                    facts.inits.entry(decl.owner.clone()).or_default().merge(decl.effects);
                    if let Some(params) = decl.unlabeled {
                        facts.unlabeled_inits.entry(decl.owner).or_default().push(params);
                    }
                }
            }
        }
        let _ = FACTS.set(facts);
    }

    fn facts() -> &'static Facts {
        FACTS
            .get()
            .unwrap_or_else(|| panic!("swift::install must run before examples convert"))
    }

    /// Every `func` and `init` declared
    /// in an `extension T { ... }` block
    /// of the `*+Extensions.swift` files.
    fn extension_decls() -> Vec<Decl> {
        let mut decls = Vec::new();
        for path in extension_files(EXTENSIONS_DIR, "+Extensions.swift") {
            let text = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("swift: cannot read {}: {e}", path.display()));
            let code = code_only(&text);
            for (owner, body) in top_level_blocks(&code, "extension") {
                extension_members(body, owner, &mut decls);
            }
        }
        decls
    }

    /// The `func` and `init` declarations directly inside one extension body.
    fn extension_members(body: &str, owner: &str, decls: &mut Vec<Decl>) {
        for (keyword, at) in top_level_words(body, &["func", "init"]) {
            let (name, after) = match keyword {
                "init" => ("new", at + keyword.len()),
                _ => read_ident(body, skip_space(body, at + keyword.len())),
            };
            let Some(open) = body[after..].find('(').map(|p| after + p) else {
                continue;
            };
            let Some(close) = closing(body, open) else {
                continue;
            };
            let modifiers = &body[body[..at].rfind('\n').map_or(0, |p| p + 1)..at];
            let tail = &body[close + 1..];
            let signature_tail = &tail[..tail.find(['{', '-']).unwrap_or(tail.len())];
            let kind = match keyword {
                "init" => CallKind::Init,
                _ if has_word(modifiers, "static") => CallKind::Static,
                _ => CallKind::Method,
            };
            decls.push(Decl {
                owner: owner.to_string(),
                name: name.to_string(),
                kind,
                effects: Effects {
                    is_async: has_word(signature_tail, "async"),
                    throws: has_word(signature_tail, "throws"),
                },
                unlabeled: unlabeled_params(&body[open + 1..close]),
            });
        }
    }

    /// The types of `_ name: Type` parameters, or `None` when one is labeled.
    fn unlabeled_params(params: &str) -> Option<Vec<String>> {
        arg_spans(params, Nesting::Types)
            .into_iter()
            .map(|span| {
                let (names, ty) = params[span].split_once(':')?;
                (names.split_whitespace().next() == Some("_"))
                    .then(|| ty.split('=').next().unwrap_or(ty).trim().to_string())
            })
            .collect()
    }

    /// `(name, body)` for each `keyword name ... { body }` at brace depth 0.
    fn top_level_blocks<'a>(code: &'a str, keyword: &str) -> Vec<(&'a str, &'a str)> {
        top_level_words(code, &[keyword])
            .into_iter()
            .filter_map(|(_, at)| {
                let (name, after) = read_ident(code, skip_space(code, at + keyword.len()));
                let open = after + code[after..].find('{')?;
                let close = closing(code, open)?;
                Some((name, &code[open + 1..close]))
            })
            .collect()
    }

    fn has_word(text: &str, word: &str) -> bool {
        text.split(|c: char| !is_ident_char(c)).any(|w| w == word)
    }

    /// Finalize the Swift output emitted by `convert(_, Lang::Swift, _)`.
    /// Takes the JS-shaped intermediate produced by the per-line emitter
    /// and applies the bulk-text swaps documented at the top of this
    /// module to land on idiomatic Swift.
    pub fn finalize(ir: &str) -> String {
        let js = swap_backticks_for_triple_quotes(ir);

        let mut out: Vec<String> = Vec::with_capacity(js.lines().count());
        let mut has_fragmentcolor_import = false;
        // Track if the previous non-empty line was a `document.createElement("canvas")` assignment.
        // We need to know the variable name so we can rewrite `createTarget(<varname>)`.
        let mut pending_canvas_var: Option<String> = None;
        let mut in_string = false;

        for raw in js.lines() {
            let starts_in_string = in_string;
            let quotes = raw.matches("\"\"\"").count();
            if quotes % 2 == 1 {
                in_string = !in_string;
            }
            // A line wholly inside a triple-quoted literal is shader source: keep it as written.
            if starts_in_string && quotes == 0 {
                out.push(raw.to_string());
                continue;
            }
            let mut line = raw.to_string();
            let trimmed: String = line.trim_start().to_string();

            if trimmed.starts_with("import ") && trimmed.contains("from \"fragmentcolor\"") {
                if !has_fragmentcolor_import {
                    out.push("import FragmentColor".to_string());
                    has_fragmentcolor_import = true;
                }
                continue;
            }

            // Drop lines that assign `document.createElement("canvas")` — iOS has no DOM.
            // Keep a comment explaining why it was dropped so the healthcheck is readable.
            if trimmed.contains("document.createElement") {
                // Extract variable name for subsequent `createTarget` rewrite.
                // Pattern: `const <varname> = document.createElement(...)` (JS) or
                //           `let <varname> = document.createElement(...)` (already-processed)
                for decl_prefix in &["const ", "let ", "var "] {
                    if let Some(var_part) = trimmed.strip_prefix(decl_prefix)
                        && let Some(eq_pos) = var_part.find('=') {
                            let var_name = var_part[..eq_pos].trim().to_string();
                            pending_canvas_var = Some(var_name);
                            break;
                        }
                }
                let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
                out.push(format!("{}// iOS: window/canvas provided by CAMetalLayer at runtime", indent));
                continue;
            }

            // Replace `createTarget(<canvas_var>)` with `createTextureTarget([800, 600])`
            // when we know the preceding line was a canvas assignment.
            // Also drop any standalone `canvas.width = ..` / `canvas.height = ..` lines.
            if let Some(ref cvar) = pending_canvas_var.clone() {
                // Drop dimension-setting lines like `canvas.width = 100`
                if trimmed.starts_with(cvar.as_str()) && (trimmed.contains(".width") || trimmed.contains(".height")) {
                    let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
                    out.push(format!("{}// (size set via createTextureTarget)", indent));
                    continue;
                }
                let pat = format!("createTarget({})", cvar);
                if line.contains(&pat) {
                    line = line.replace(&pat, "createTextureTarget([800, 600])");
                    pending_canvas_var = None;
                }
                // Also handle trailing comma / paren variants.
                let pat2 = format!("createTarget({},", cvar);
                if line.contains(&pat2) {
                    line = line.replace(&pat2, "createTextureTarget([800, 600],");
                    pending_canvas_var = None;
                }
            }

            out.push(rewrite_line(&line, starts_in_string));
        }

        // Multi-line trailing comma strip. The convert layer emits
        // calls like
        //
        //   try mesh.addVertex(
        //       try Vertex.pbr(...).set("uv0", [0.5, 1.0]),
        //   )
        //
        // Swift parses the dangling `,\n)` as the empty-element tuple
        // shape and bails with "unexpected ',' separator". Work on the
        // joined text so a comma and its `)` may sit on different lines:
        // replace any `,<whitespace>)` with `<whitespace>)`.
        let joined = out.join("\n");
        let joined = strip_trailing_comma_multiline(&joined);
        // Also handle two-pass duplicate `let <name>` declarations
        // inside a single static-func scope. Two `let shader = ...` in
        // the same scope are a Swift compile error; rename the second
        // and onwards to `<name>2`, `<name>3`, etc.
        rename_duplicate_let(&joined)
    }

    /// Cross-line scan that removes a `,` immediately followed by
    /// whitespace (possibly newlines) and a `)`. Strings and triple-
    /// quoted blocks are skipped so WGSL inside `"""..."""` literals is
    /// left alone.
    fn strip_trailing_comma_multiline(src: &str) -> String {
        let chars: Vec<char> = src.chars().collect();
        let mut out: Vec<char> = Vec::with_capacity(chars.len());
        let mut in_dq = false;
        let mut in_tq = false;
        let mut i = 0usize;
        while i < chars.len() {
            if i + 2 < chars.len()
                && chars[i] == '"'
                && chars[i + 1] == '"'
                && chars[i + 2] == '"'
            {
                in_tq = !in_tq;
                out.push('"');
                out.push('"');
                out.push('"');
                i += 3;
                continue;
            }
            if !in_tq && chars[i] == '"' {
                in_dq = !in_dq;
                out.push('"');
                i += 1;
                continue;
            }
            if !in_dq && !in_tq && chars[i] == ',' {
                // Look past whitespace (incl. newlines) for the next
                // non-whitespace char. If it's `)`, drop the comma.
                let mut j = i + 1;
                while j < chars.len() && chars[j].is_whitespace() {
                    j += 1;
                }
                if j < chars.len() && chars[j] == ')' {
                    i += 1;
                    continue;
                }
            }
            out.push(chars[i]);
            i += 1;
        }
        out.into_iter().collect()
    }

    /// Rename duplicate `let <name>` declarations inside a single
    /// static-func body. Swift's enum/static-func scope can't shadow
    /// existing bindings — two `let shader = ...` lines collide with
    /// "invalid redeclaration of 'shader'". We track names per
    /// `static func` and rewrite the second occurrence (and later)
    /// to `<name>2`, `<name>3`, … within the same function body.
    ///
    /// Conservative shape: only matches lines whose trimmed form
    /// starts with `let <ident> = ` or `let <ident>: …`. Names declared
    /// inside `if let`/`guard let`/destructuring `let (...)` are left
    /// alone (those forms don't appear in our generated example bodies).
    fn rename_duplicate_let(src: &str) -> String {
        let mut out: Vec<String> = Vec::with_capacity(src.lines().count());
        let mut declared: std::collections::HashMap<String, u32> =
            std::collections::HashMap::new();
        let mut in_tq = false;
        for raw in src.lines() {
            // Decide first whether the line *started* inside a triple-quoted
            // string. Then toggle the state for subsequent lines. A line that
            // opens `"""` carries valid Swift code before the marker, so we
            // still want to process it.
            let was_in_tq = in_tq;
            let tq_count = raw.matches("\"\"\"").count();
            if tq_count % 2 == 1 {
                in_tq = !in_tq;
            }
            if was_in_tq {
                out.push(raw.to_string());
                continue;
            }
            // Reset declared map at every `static func` boundary so each
            // example body starts fresh.
            if raw.trim_start().starts_with("static func ") {
                declared.clear();
            }
            let trimmed = raw.trim_start();
            // Only handle `let <ident> = ...` or `let <ident>: ...`.
            if let Some(rest) = trimmed.strip_prefix("let ") {
                // Ident is the run of identifier chars.
                let end = rest
                    .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                    .unwrap_or(rest.len());
                let ident = &rest[..end];
                let after = rest[end..].trim_start();
                let is_decl = after.starts_with('=') || after.starts_with(':');
                if !ident.is_empty() && is_decl {
                    let count = declared.entry(ident.to_string()).or_insert(0);
                    *count += 1;
                    if *count > 1 {
                        let n = *count;
                        let new_ident = format!("{}{}", ident, n);
                        let indent_len = raw.len() - trimmed.len();
                        let indent = &raw[..indent_len];
                        // Reconstruct: indent + "let <new_ident>" + rest[end..]
                        out.push(format!("{}let {}{}", indent, new_ident, &rest[end..]));
                        continue;
                    }
                }
            }
            out.push(raw.to_string());
        }
        out.join("\n")
    }

    /// Swift and Kotlin both use `"""..."""` for multi-line strings, whereas
    /// the JS output uses backticks. The two formats are otherwise
    /// interchangeable for our examples (no JS template-literal interpolation
    /// is emitted by `convert.rs`).
    pub(crate) fn swap_backticks_for_triple_quotes(js: &str) -> String {
        js.replace('`', "\"\"\"")
    }

    /// Rewrite one line of the intermediate,
    /// leaving `try` and `await` out of a line
    /// that starts inside a `"""` string literal.
    fn rewrite_line(src: &str, in_string: bool) -> String {
        let mut out = src.to_string();

        // Trailing `;` strip (preserve indentation).
        if let Some(stripped) = out.trim_end().strip_suffix(';') {
            let leading_len = out.len() - out.trim_start().len();
            let leading = &src[..leading_len];
            out = format!("{}{}", leading, stripped.trim_start());
        }
        if !in_string {
            out = drop_semicolon_before_comment(&out);
        }

        out = drop_new_keyword(&out);
        out = replace_leading_keyword(&out, "const ", "let ");
        out = prepend_try_to_await(&out);
        out = replace_whole_word(&out, "null", "nil");
        out = rewrite_array_fill(&out);
        out = swap_single_quoted_strings(&out);

        // Swift-specific rewrites beyond the JS baseline:

        // 9. `TextureRegion` → `TextureRegionMobile`, the mobile-only type that stands in for it.
        for s in &facts().stand_ins {
            out = replace_whole_word(&out, &s.ty, &s.stand_in);
        }

        // 10. `x..y` → `x..<y` and `x..=y` → `x...y`.
        out = swift_ranges(&out);

        // 11. Enum case first-letter lowercasing: `.Rgba` → `.rgba`, `.Rgba8UnormSrgb` → `.rgba8UnormSrgb`.
        //     Applies to patterns like `TextureFormat.XYZ` or `.XYZ` in call argument position.
        out = lowercase_enum_cases(&out);

        // 12. `.unwrap()` after a throwing init: rewrite `Foo(...).unwrap()` → `try! Foo(...)`.
        out = rewrite_unwrap(&out);

        // 13. `Vertex.new([...])` or `Instance.new()`, a nested Rust `::new`,
        //     becomes a Swift init call.
        out = rewrite_dot_new(&out);

        // 14. `try` / `await` for calls the JS output makes synchronously.
        if !in_string {
            out = insert_effects(&out);
        }

        // Strip Rust slice notation `identifier[...]` → `identifier` when `...`
        // is the only content inside brackets (a full-range slice).
        // e.g. `image[...]` → `image`, `pixels.asSlice()` → `pixels`.
        out = strip_rust_slice_notation(&out);

        // Inline `Vertex.UV0` (and the rest of the SCREAMING_SNAKE
        // attribute-name constants) as the bare string literal value.
        // Uniffi has no equivalent of pyo3 classattr or Kotlin static const
        // to surface the Rust-side `pub const UV0: &str = "uv0"` on the
        // Swift binding; the transpiler resolves the lookup here so
        // example code keeps reading `Vertex::UV0` on the Rust side.
        out = inline_vertex_attr_constants(&out);

        out
    }

    /// Inline associated string consts such as `Vertex.UV0`, which uniffi
    /// can't surface on Swift.
    fn inline_vertex_attr_constants(line: &str) -> String {
        crate::reflect::inline_str_consts(line, &facts().consts, crate::convert::quoted)
    }

    /// Rust ranges in Swift: `a..b` → `a..<b` and `a..=b` → `a...b`.
    /// Strings, comments and the full-range slice `[..]` stay as written.
    fn swift_ranges(line: &str) -> String {
        let code = code_only(line);
        let mut out = String::with_capacity(line.len() + 4);
        let mut last = 0;
        let mut at = 0;
        while let Some(found) = code[at..].find("..").map(|p| at + p) {
            let after = &code[found + 2..];
            let (swift, len) = if after.starts_with('.') {
                (None, 3)
            } else if code[..found].ends_with('[') && after.starts_with(']') {
                (None, 2)
            } else if after.starts_with('=') {
                (Some("..."), 3)
            } else {
                (Some("..<"), 2)
            };
            if let Some(swift) = swift {
                out.push_str(&line[last..found]);
                out.push_str(swift);
                last = found + len;
            }
            at = found + len;
        }
        out.push_str(&line[last..]);
        out
    }

    /// Lowercase the first letter of an enum case after a dot.
    /// Handles patterns like `TextureFormat.Rgba` → `TextureFormat.rgba`,
    /// `.Rgba8UnormSrgb` → `.rgba8UnormSrgb`, etc.
    /// Only applies when the letter after `.` is uppercase and NOT inside a string.
    fn lowercase_enum_cases(line: &str) -> String {
        let chars: Vec<char> = line.chars().collect();
        let mut out = String::with_capacity(line.len());
        let mut i = 0usize;
        let mut in_string = false;
        let mut in_triple = false;
        while i < chars.len() {
            // Track triple-quote strings.
            if !in_string && i + 2 < chars.len() && chars[i] == '"' && chars[i+1] == '"' && chars[i+2] == '"' {
                in_triple = !in_triple;
                out.push('"'); out.push('"'); out.push('"');
                i += 3;
                continue;
            }
            // Track double-quote strings.
            if !in_triple && chars[i] == '"' {
                in_string = !in_string;
                out.push('"');
                i += 1;
                continue;
            }
            if !in_string && !in_triple && chars[i] == '.' {
                // Check if next char is uppercase and we're not at `...` or after a number.
                let prev_is_digit = i > 0 && chars[i - 1].is_ascii_digit();
                if !prev_is_digit && i + 1 < chars.len() && chars[i + 1].is_ascii_uppercase() {
                    // Peek ahead: the char after the uppercase letter must be alphanumeric
                    // (this is a PascalCase enum case, not e.g. `.utf8` or `.init`).
                    if i + 2 < chars.len() && (chars[i + 2].is_ascii_alphanumeric() || chars[i + 2] == '_') {
                        // Read the full identifier so we can distinguish PascalCase
                        // enum cases (Rgba8UnormSrgb) from SCREAMING_SNAKE constants
                        // (UV0, NORMAL, COLOR0, FRAC_PI_4). Only the former should be
                        // lowercased here; the latter are class-level constants whose
                        // names round-trip verbatim.
                        let mut end = i + 1;
                        while end < chars.len() && (chars[end].is_ascii_alphanumeric() || chars[end] == '_') {
                            end += 1;
                        }
                        let ident: String = chars[i + 1..end].iter().collect();
                        let is_screaming = !ident.is_empty()
                            && ident.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');
                        if !is_screaming {
                            out.push('.');
                            // Lowercase the first letter.
                            out.push(chars[i + 1].to_ascii_lowercase());
                            i += 2;
                            continue;
                        }
                    }
                }
            }
            out.push(chars[i]);
            i += 1;
        }
        out
    }

    /// Rewrite `SomeType(...).unwrap()` → `try! SomeType(...)`.
    /// In Rust, `Shader::new(src).unwrap()` is common; in Swift the
    /// equivalent is `try! Shader(src)` (the init is `throws`).
    fn rewrite_unwrap(line: &str) -> String {
        let _trimmed = line.trim_start();
        // Look for `.unwrap()` at end of expression.
        if !line.contains(".unwrap()") {
            return line.to_string();
        }
        let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
        let body = line.trim_start();
        // Strip `.unwrap()` and prepend `try! ` (or `try ` if already in try context).
        let body = body.replace(".unwrap()", "");
        // Detect assignment prefix `let x = ...` or plain statement.
        if let Some(rest) = body.strip_prefix("let ")
            && let Some(eq) = rest.find('=') {
                let var_part = &rest[..eq + 1]; // "varname ="
                let rhs = rest[eq + 1..].trim();
                return format!("{}let {} try! {}", indent, var_part, rhs);
            }
        // For plain statements like `foo.bar().unwrap()` → `try! foo.bar()`.
        if !body.starts_with("try") {
            format!("{}try! {}", indent, body.trim_start())
        } else {
            format!("{}{}", indent, body)
        }
    }

    /// `T.new(args)` → `T(args)`, nested calls too,
    /// wherever [`init_call`] finds an initializer to call.
    /// Otherwise `T.new` stays, as for the static `Shader.new([...])`.
    fn rewrite_dot_new(line: &str) -> String {
        let mut out = String::with_capacity(line.len());
        let mut rest = line;
        while let Some(at) = rest.find(".new(") {
            let head = &rest[..at];
            let before = head.trim_end_matches(is_ident_char);
            let ty = &head[before.len()..];
            let open = at + ".new".len();
            let call = closing(rest, open)
                .filter(|_| !before.ends_with('.'))
                .and_then(|close| Some((init_call(ty, &rest[open + 1..close])?, close)));
            match call {
                Some((call, close)) => {
                    out.push_str(before);
                    out.push_str(&call);
                    rest = &rest[close + 1..];
                }
                None => {
                    out.push_str(&rest[..open]);
                    rest = &rest[open..];
                }
            }
        }
        out.push_str(rest);
        out
    }

    /// `T(args)` for `T.new(args)`, with `try` when an initializer of `T` throws.
    /// `None` unless an unlabeled one takes as many arguments, arrays at array literals.
    fn init_call(ty: &str, inner: &str) -> Option<String> {
        let facts = facts();
        let inits = facts.unlabeled_inits.get(ty)?;
        let args = split_args(inner);
        let fits = inits.iter().any(|params| {
            params.len() == args.len()
                && args
                    .iter()
                    .zip(params)
                    .all(|(arg, param)| !arg.starts_with('[') || param.starts_with('['))
        });
        let throws = facts.inits.get(ty).is_some_and(|fx| fx.throws);
        let prefix = if throws { "try " } else { "" };
        fits.then(|| format!("{prefix}{ty}({})", rewrite_dot_new(inner)))
    }

    /// Strip Rust slice / `as_slice()` notation that has no equivalent in Swift:
    /// - `identifier[..]`   → `identifier`  (full-range borrow `&x[..]` after `&`-strip)
    /// - `.asSlice()`        → ``            (Rust `.as_slice()` method — just drop it)
    fn strip_rust_slice_notation(line: &str) -> String {
        // 1. Remove `.asSlice()` (comes from Rust `.as_slice()` transliterated by to_js).
        let line = line.replace(".asSlice()", "");
        // 2. Remove the full-range slice: `ident[..]` → `ident`.
        line.replace("[..]", "")
    }

    /// Prefix a statement with the `try` and `await` its calls need.
    /// A `let`, `var` or `return` gets them in front of its value instead.
    /// Lines that already carry either, and lines led by another keyword, stay as written.
    fn insert_effects(line: &str) -> String {
        let trimmed = line.trim_start();
        let skip = ["try ", "try!", "//", "/*", "*", "\""];
        if skip.iter().any(|s| trimmed.starts_with(s)) || trimmed.contains("await ") {
            return line.to_string();
        }
        let indent = &line[..line.len() - trimmed.len()];
        let Some((lead, expr)) = effect_site(trimmed) else {
            return line.to_string();
        };
        if expr.starts_with("try ") || expr.starts_with("try!") {
            return line.to_string();
        }
        match call_effects(expr).keyword() {
            Some(effect) => format!("{indent}{lead}{effect} {expr}"),
            None => line.to_string(),
        }
    }

    /// Split a statement into the text that stays in front of `try` / `await`
    /// and the expression they cover, or `None` when they can't go in.
    fn effect_site(statement: &str) -> Option<(String, &str)> {
        for keyword in ["let ", "var "] {
            if let Some(rest) = statement.strip_prefix(keyword)
                && let Some(eq) = rest.find('=')
            {
                return Some((format!("{keyword}{} ", &rest[..=eq]), rest[eq + 1..].trim_start()));
            }
        }
        let (word, end) = read_ident(statement, 0);
        if word == "return" {
            let value = statement[end..].trim_start();
            return (!value.is_empty()).then(|| ("return ".to_string(), value));
        }
        let leads = statement.starts_with(|c: char| c.is_alphanumeric() || c == '_' || c == '(');
        (leads && !STATEMENT_KEYWORDS.contains(&word)).then(|| (String::new(), statement))
    }

    /// The merged effects of every call in `code`.
    /// `T.name(` with a type name `T` is a static call and `T(` an initializer,
    /// while any other `.name(` is an instance method known only by its name.
    fn call_effects(code: &str) -> Effects {
        let facts = facts();
        let mut effects = Effects::default();
        let mut at = 0;
        while let Some(c) = code[at..].chars().next() {
            if c == '"' {
                at += string_end(&code[at..]);
                continue;
            }
            let head = &code[..at];
            if !(c.is_ascii_alphabetic() || c == '_') || head.ends_with(is_ident_char) {
                at += c.len_utf8();
                continue;
            }
            let (name, end) = read_ident(code, at);
            if code[end..].starts_with('(') {
                let found = match head.strip_suffix('.') {
                    Some(before) => {
                        let receiver = &before[before.trim_end_matches(is_ident_char).len()..];
                        let bare = !before[..before.len() - receiver.len()].ends_with('.');
                        if bare && is_type_name(receiver) {
                            facts.statics.get(&(receiver.to_string(), name.to_string()))
                        } else {
                            facts.methods.get(name)
                        }
                    }
                    None if is_type_name(name) => facts.inits.get(name),
                    None => None,
                };
                if let Some(found) = found {
                    effects.merge(*found);
                }
            }
            at = end;
        }
        effects
    }

    fn is_type_name(ident: &str) -> bool {
        ident.starts_with(|c: char| c.is_ascii_uppercase())
    }

    /// `Array(N).fill(expr)` (the JS shape `convert.rs` emits for
    /// Rust array-repeat) → `Array(repeating: expr, count: N)`. Bracket-
    /// balanced walk so nested expressions inside the args don't trip it.
    fn rewrite_array_fill(line: &str) -> String {
        let needle = "Array(";
        let mut out = String::with_capacity(line.len());
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0usize;
        while i < chars.len() {
            let rem: String = chars[i..].iter().collect();
            if rem.starts_with(needle) {
                // Walk to matching `)` for `Array(...)`.
                let arg_start = i + needle.len();
                let mut depth = 0i32;
                let mut arg_close: Option<usize> = None;
                let mut k = arg_start;
                while k < chars.len() {
                    match chars[k] {
                        '(' => depth += 1,
                        ')' => {
                            if depth == 0 {
                                arg_close = Some(k);
                                break;
                            }
                            depth -= 1;
                        }
                        _ => {}
                    }
                    k += 1;
                }
                if let Some(arg_close) = arg_close {
                    // Expect `.fill(` immediately after.
                    let after: String = chars[arg_close + 1..].iter().collect();
                    if let Some(stripped) = after.strip_prefix(".fill(") {
                        let fill_arg_chars: Vec<char> = stripped.chars().collect();
                        let mut depth2 = 0i32;
                        let mut fill_close: Option<usize> = None;
                        let mut m = 0usize;
                        while m < fill_arg_chars.len() {
                            match fill_arg_chars[m] {
                                '(' => depth2 += 1,
                                ')' => {
                                    if depth2 == 0 {
                                        fill_close = Some(m);
                                        break;
                                    }
                                    depth2 -= 1;
                                }
                                _ => {}
                            }
                            m += 1;
                        }
                        if let Some(fc) = fill_close {
                            let count: String = chars[arg_start..arg_close].iter().collect();
                            let expr: String = fill_arg_chars[..fc].iter().collect();
                            out.push_str(&format!(
                                "Array(repeating: {}, count: {})",
                                expr.trim(),
                                count.trim()
                            ));
                            // Advance past the closing `)` of `.fill(...)`.
                            // arg_close + 1 is the first char of `.fill(`,
                            // so absolute pos of fill's `)` is arg_close + 1 + ".fill(".len() + fc.
                            i = arg_close + 1 + ".fill(".len() + fc + 1;
                            continue;
                        }
                    }
                }
            }
            out.push(chars[i]);
            i += 1;
        }
        out
    }

    /// Convert JS-style single-quoted string literals (`'foo'`) to Swift
    /// double-quoted (`"foo"`). Skips swaps when we're already inside a
    /// `"..."` or `"""..."""` so apostrophes in WGSL comments survive.
    fn swap_single_quoted_strings(line: &str) -> String {
        let chars: Vec<char> = line.chars().collect();
        let mut out = String::with_capacity(line.len());
        let mut i = 0usize;
        let mut in_dq = false;
        let mut in_tq = false;
        while i < chars.len() {
            // Triple-quote detection
            if !in_dq
                && i + 2 < chars.len()
                && chars[i] == '"'
                && chars[i + 1] == '"'
                && chars[i + 2] == '"'
            {
                in_tq = !in_tq;
                out.push_str("\"\"\"");
                i += 3;
                continue;
            }
            if !in_tq && !in_dq && chars[i] == '"' {
                in_dq = true;
                out.push('"');
                i += 1;
                continue;
            }
            if in_dq && chars[i] == '"' {
                in_dq = false;
                out.push('"');
                i += 1;
                continue;
            }
            // Single quote → swap if outside any string context AND the
            // matching quote sits on the same line and the content has
            // no internal `"`.
            if !in_dq && !in_tq && chars[i] == '\''
                && let Some(end) = chars[i + 1..].iter().position(|c| *c == '\'') {
                    let inner: String = chars[i + 1..i + 1 + end].iter().collect();
                    if !inner.contains('"') && !inner.contains('\n') {
                        out.push('"');
                        out.push_str(&inner);
                        out.push('"');
                        i += 1 + end + 1;
                        continue;
                    }
                }
            out.push(chars[i]);
            i += 1;
        }
        out
    }

    fn replace_leading_keyword(line: &str, from: &str, to: &str) -> String {
        let leading_len = line.len() - line.trim_start().len();
        let (indent, rest) = line.split_at(leading_len);
        if let Some(after) = rest.strip_prefix(from) {
            format!("{}{}{}", indent, to, after)
        } else {
            line.to_string()
        }
    }

    /// Remove `new ` used as a constructor prefix: `new Type(...)` → `Type(...)`.
    /// Matches only a standalone `new` in code, never in strings or comments.
    fn drop_new_keyword(line: &str) -> String {
        let code = code_only(line);
        let mut out = String::with_capacity(line.len());
        let (mut copied, mut search) = (0, 0);
        while let Some(found) = code[search..].find("new ") {
            let at = search + found;
            let left_ok = code[..at].chars().next_back().is_none_or(|c| !is_ident_char(c));
            let right_ok = code[at + 4..].chars().next().is_some_and(|c| c.is_alphabetic() || c == '_');
            if left_ok && right_ok {
                out.push_str(&line[copied..at]);
                copied = at + 4;
            }
            search = at + 4;
        }
        out.push_str(&line[copied..]);
        out
    }

    /// Prepend `try ` before the first `await` on a line when appropriate.
    fn prepend_try_to_await(line: &str) -> String {
        if !line.contains("await ") || line.contains("try await") {
            return line.to_string();
        }
        let idx = match line.find("await ") {
            Some(i) => i,
            None => return line.to_string(),
        };
        if idx > 0 {
            let prev = line.as_bytes()[idx - 1] as char;
            if is_ident_char(prev) {
                return line.to_string();
            }
        }
        let (left, right) = line.split_at(idx);
        format!("{}try {}", left, right)
    }

    fn replace_whole_word(src: &str, from: &str, to: &str) -> String {
        let bytes: Vec<char> = src.chars().collect();
        let needle: Vec<char> = from.chars().collect();
        let mut out = String::with_capacity(src.len());
        let mut i = 0usize;
        while i < bytes.len() {
            let end = i + needle.len();
            if end <= bytes.len() && bytes[i..end] == needle[..] {
                let left_ok = i == 0 || !is_ident_char(bytes[i - 1]);
                let right_ok = end == bytes.len() || !is_ident_char(bytes[end]);
                if left_ok && right_ok {
                    out.push_str(to);
                    i = end;
                    continue;
                }
            }
            out.push(bytes[i]);
            i += 1;
        }
        out
    }
}
