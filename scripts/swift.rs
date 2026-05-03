mod swift {
    //! Swift transpilation.
    //!
    //! We start from the JS output (see `convert::to_swift`) and apply a
    //! handful of lexical swaps to produce idiomatic Swift. This deliberately
    //! avoids re-implementing the full Rust → language pipeline a second
    //! time — JS already does all the hard work (method name camelizing,
    //! `::` → `.`, collapsing `impl Into<T>` constructors, etc.).
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
    //! 9. `TextureRegion` → `TextureRegionMobile` (uniffi-side rename).
    //! 10. `x..y` range operator → `x...y` (Swift uses `...` not `..`).
    //! 11. `.Rgba` / `.Rgba8UnormSrgb` enum case first-letter lowercasing.
    //! 12. `.unwrap()` on a throwing init → `try!` prefix.
    //! 13. `Vertex.new([...])` → `Vertex([...])` (Rust .new() → Swift init).
    //! 14. `try` insertion for known throwing methods called without `await`
    //!     (uniffi marks most methods `throws`; JS equivalents are synchronous).

    pub fn js_to_swift(js: &str) -> String {
        let js = swap_backticks_for_triple_quotes(js);

        let mut out: Vec<String> = Vec::with_capacity(js.lines().count());
        let mut has_fragmentcolor_import = false;
        // Track if the previous non-empty line was a `document.createElement("canvas")` assignment.
        // We need to know the variable name so we can rewrite `createTarget(<varname>)`.
        let mut pending_canvas_var: Option<String> = None;

        for raw in js.lines() {
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
                    if let Some(var_part) = trimmed.strip_prefix(decl_prefix) {
                        if let Some(eq_pos) = var_part.find('=') {
                            let var_name = var_part[..eq_pos].trim().to_string();
                            pending_canvas_var = Some(var_name);
                            break;
                        }
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

            out.push(rewrite_line(&line));
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

    fn rewrite_line(src: &str) -> String {
        let mut out = src.to_string();

        // Trailing `;` strip (preserve indentation).
        if let Some(stripped) = out.trim_end().strip_suffix(';') {
            let leading_len = out.len() - out.trim_start().len();
            let leading = &src[..leading_len];
            out = format!("{}{}", leading, stripped.trim_start());
        }

        out = drop_new_keyword(&out);
        out = replace_leading_keyword(&out, "const ", "let ");
        out = prepend_try_to_await(&out);
        out = replace_whole_word(&out, "null", "nil");
        out = rewrite_array_fill(&out);
        out = swap_single_quoted_strings(&out);

        // Swift-specific rewrites beyond the JS baseline:

        // 8. `TextureRegion` → `TextureRegionMobile` (uniffi-side rename for FFI compat).
        out = replace_whole_word(&out, "TextureRegion", "TextureRegionMobile");

        // 9. `x..y` Rust exclusive-range operator → `x...y` (Swift closed range `...`).
        //    Only rewrite `..` when NOT preceded or followed by `.` (avoids `...`).
        out = fix_rust_range_operator(&out);

        // 10. Enum case first-letter lowercasing: `.Rgba` → `.rgba`, `.Rgba8UnormSrgb` → `.rgba8UnormSrgb`.
        //     Applies to patterns like `TextureFormat.XYZ` or `.XYZ` in call argument position.
        out = lowercase_enum_cases(&out);

        // 11. `.unwrap()` after a throwing init — rewrite `Foo(...).unwrap()` → `try! Foo(...)`.
        out = rewrite_unwrap(&out);

        // 12. `Vertex.new([...])` or `Instance.new()` — Rust `.new()` call → Swift init/static.
        //     `.new()` with no args → `()` (already handled by drop_new_keyword for `new Type()` form,
        //     but `.new([...])` is a chained method not a constructor prefix).
        out = rewrite_dot_new(&out);

        // 13. Add `try` to known throwing method calls not already prefixed with try/await.
        //     The uniffi-generated Swift API marks many synchronous methods `throws` that
        //     appear as plain calls in the JS output.
        out = insert_try_for_throws(&out);

        out
    }

    /// Fix Rust `x..y` exclusive-range into Swift `x...y` (closed range).
    /// We use a simple scan: `..` that is not already `...` becomes `...`.
    fn fix_rust_range_operator(line: &str) -> String {
        let chars: Vec<char> = line.chars().collect();
        let mut out = String::with_capacity(line.len() + 4);
        let mut i = 0usize;
        while i < chars.len() {
            // Look for `..` that is not already `...`
            if i + 1 < chars.len() && chars[i] == '.' && chars[i + 1] == '.' {
                if i + 2 < chars.len() && chars[i + 2] == '.' {
                    // Already `...`, pass through.
                    out.push('.');
                    out.push('.');
                    out.push('.');
                    i += 3;
                } else {
                    // Bare `..` → `...`
                    out.push('.');
                    out.push('.');
                    out.push('.');
                    i += 2;
                }
                continue;
            }
            out.push(chars[i]);
            i += 1;
        }
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
                        out.push('.');
                        // Lowercase the first letter.
                        out.push(chars[i + 1].to_ascii_lowercase());
                        i += 2;
                        continue;
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
        let trimmed = line.trim_start();
        // Look for `.unwrap()` at end of expression.
        if !line.contains(".unwrap()") {
            return line.to_string();
        }
        let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
        let body = line.trim_start();
        // Strip `.unwrap()` and prepend `try! ` (or `try ` if already in try context).
        let body = body.replace(".unwrap()", "");
        // Detect assignment prefix `let x = ...` or plain statement.
        if let Some(rest) = body.strip_prefix("let ") {
            if let Some(eq) = rest.find('=') {
                let var_part = &rest[..eq + 1]; // "varname ="
                let rhs = rest[eq + 1..].trim();
                return format!("{}let {} try! {}", indent, var_part, rhs);
            }
        }
        // For plain statements like `foo.bar().unwrap()` → `try! foo.bar()`.
        if !body.starts_with("try") {
            format!("{}try! {}", indent, body.trim_start())
        } else {
            format!("{}{}", indent, body)
        }
    }

    /// Rewrite `.new(args)` chained calls on known types.
    /// e.g. `Vertex.new([...])` → `try Vertex([...])`, `Instance.new()` → `Instance()`.
    /// This is different from `drop_new_keyword` which handles `new Type(...)` prefix form.
    fn rewrite_dot_new(line: &str) -> String {
        // Types where `.new(...)` → `try Type(...)` (throwing inits).
        const THROWING_TYPES_WITH_DOT_NEW: &[&str] = &["Vertex"];
        // Types where `.new(...)` → `Type(...)` (non-throwing).
        const NON_THROWING_TYPES_WITH_DOT_NEW: &[&str] = &["Instance"];
        let mut out = line.to_string();
        for ty in THROWING_TYPES_WITH_DOT_NEW {
            let pat = format!("{}.new(", ty);
            while let Some(pos) = out.find(&pat) {
                let arg_start = pos + pat.len();
                let chars: Vec<char> = out.chars().collect();
                let mut depth = 1i32;
                let mut k = arg_start;
                while k < chars.len() && depth > 0 {
                    match chars[k] {
                        '(' => depth += 1,
                        ')' => depth -= 1,
                        _ => {}
                    }
                    k += 1;
                }
                let args: String = out[arg_start..k - 1].to_string();
                let replacement = if args.is_empty() {
                    format!("try {}()", ty)
                } else {
                    format!("try {}({})", ty, args)
                };
                let new_out = format!("{}{}{}", &out[..pos], replacement, &out[k..]);
                out = new_out;
            }
        }
        for ty in NON_THROWING_TYPES_WITH_DOT_NEW {
            let pat = format!("{}.new(", ty);
            while let Some(pos) = out.find(&pat) {
                let arg_start = pos + pat.len();
                let chars: Vec<char> = out.chars().collect();
                let mut depth = 1i32;
                let mut k = arg_start;
                while k < chars.len() && depth > 0 {
                    match chars[k] {
                        '(' => depth += 1,
                        ')' => depth -= 1,
                        _ => {}
                    }
                    k += 1;
                }
                let args: String = out[arg_start..k - 1].to_string();
                let replacement = if args.is_empty() {
                    format!("{}()", ty)
                } else {
                    format!("{}({})", ty, args)
                };
                let new_out = format!("{}{}{}", &out[..pos], replacement, &out[k..]);
                out = new_out;
            }
        }
        out
    }

    /// Insert `try ` before calls to methods known to be `throws` in the uniffi API
    /// when they appear without any existing `try` or `await` prefix on the line.
    ///
    /// Strategy: if the line (trimmed) starts with one of the known throwing method
    /// names (as a standalone call), or if the RHS of a `let x = method(...)` is
    /// a known throwing call, prepend `try`.
    ///
    /// We keep the list conservative: only methods that are demonstrably `throws`
    /// in the extension files and that appear as plain calls in the generated examples.
    fn insert_try_for_throws(line: &str) -> String {
        // Methods that are `throws` in the Swift extensions but appear as plain calls
        // in the JS/Swift transpilation output (no `await`, so `prepend_try_to_await`
        // doesn't catch them).
        const THROWING_METHODS: &[&str] = &[
            // Mesh
            ".addVertex(",
            ".addVertices(",
            ".fromVertices(",
            // Shader
            ".addMesh(",
            ".validateMesh(",
            ".set(",
            ".get(",
            ".removeMesh(",
            // Pass
            ".addMeshToShader(",
            ".addTarget(",
            ".addDepthTarget(",
            ".require(",
            ".setClearColor(",
            // Renderer (sync variants)
            ".render(",
            ".unregisterTexture(",
            ".waitIdle(",
            // Texture
            ".write(",
            ".writeRegion(",
            ".setSamplerOptions(",
        ];
        // Constructors (type inits) that are `throws`.
        // Matched as `let x = TypeName(` or standalone `TypeName(`.
        const THROWING_CTORS: &[&str] = &[
            "Shader(",
        ];

        let trimmed = line.trim_start();

        // Skip lines already starting with `try`, `await`, `//`, string literals, etc.
        if trimmed.starts_with("try ")
            || trimmed.starts_with("try!")
            || trimmed.starts_with("//")
            || trimmed.starts_with("/*")
            || trimmed.starts_with("*")
            || trimmed.starts_with("\"")
        {
            return line.to_string();
        }

        let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();

        // Case 1: `let x = <throwing_call>` or `var x = <throwing_call>`.
        for prefix in &["let ", "var "] {
            if let Some(rest) = trimmed.strip_prefix(prefix) {
                if let Some(eq_pos) = rest.find('=') {
                    let rhs = rest[eq_pos + 1..].trim_start();
                    let var_part = &rest[..eq_pos + 1]; // "varname ="
                    // Check throwing method calls.
                    for meth in THROWING_METHODS {
                        if rhs.contains(meth) && !rhs.starts_with("try ") && !rhs.contains("await ") {
                            let new_rhs = insert_try_before_method(rhs, meth);
                            return format!("{}{}{} {}", indent, prefix, var_part, new_rhs);
                        }
                    }
                    // Check throwing constructors.
                    for ctor in THROWING_CTORS {
                        if rhs.starts_with(ctor) && !rhs.starts_with("try ") && !rhs.contains("await ") {
                            return format!("{}{}{} try {}", indent, prefix, var_part, rhs);
                        }
                    }
                }
            }
        }

        // Case 2: Standalone statement that is a throwing method call.
        // Pattern: `obj.method(...)` or `Type.method(...)` as the whole line.
        for meth in THROWING_METHODS {
            if trimmed.contains(meth) && !trimmed.starts_with("try ") && !trimmed.contains("await ") {
                // Make sure this is a statement (not inside a string or a declaration).
                // Heuristic: if the trimmed line starts with a method call chain.
                if trimmed.starts_with(|c: char| c.is_alphanumeric() || c == '_' || c == '(') {
                    return format!("{}try {}", indent, trimmed);
                }
            }
        }

        line.to_string()
    }

    /// Insert `try ` directly before a method reference within an expression.
    fn insert_try_before_method(rhs: &str, _meth: &str) -> String {
        // Simple heuristic: if rhs doesn't start with `try`, prepend it.
        if rhs.starts_with("try ") {
            rhs.to_string()
        } else {
            format!("try {}", rhs)
        }
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
            if !in_dq && !in_tq && chars[i] == '\'' {
                if let Some(end) = chars[i + 1..].iter().position(|c| *c == '\'') {
                    let inner: String = chars[i + 1..i + 1 + end].iter().collect();
                    if !inner.contains('"') && !inner.contains('\n') {
                        out.push('"');
                        out.push_str(&inner);
                        out.push('"');
                        i += 1 + end + 1;
                        continue;
                    }
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
    /// Matches only when `new` appears as a standalone token.
    fn drop_new_keyword(line: &str) -> String {
        let chars: Vec<char> = line.chars().collect();
        let mut out = String::with_capacity(line.len());
        let mut i = 0usize;
        while i < chars.len() {
            if chars[i..].starts_with(&['n', 'e', 'w', ' ']) {
                let left_ok = i == 0 || !is_ident_char(chars[i - 1]);
                let right_ok =
                    i + 4 < chars.len() && (chars[i + 4].is_alphabetic() || chars[i + 4] == '_');
                if left_ok && right_ok {
                    i += 4; // skip "new "
                    continue;
                }
            }
            out.push(chars[i]);
            i += 1;
        }
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

    fn is_ident_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }
}
