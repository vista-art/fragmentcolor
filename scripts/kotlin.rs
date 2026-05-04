mod kotlin {
    //! Kotlin transpilation.
    //!
    //! Same approach as `swift.rs`: post-process the JS output. Kotlin
    //! differs from Swift in:
    //!   - `null` stays as `null` (Kotlin keyword).
    //!   - `const` → `val` (not `let`).
    //!   - Async methods are `suspend` functions; callers drop `await`
    //!     entirely and run inside `runBlocking { ... }` or another
    //!     coroutine scope (the healthcheck wrapper handles this).
    //!   - Import becomes `import org.fragmentcolor.*`.
    //!   - Trailing semicolons are optional and we strip them.

    pub fn js_to_kotlin(js: &str) -> String {
        let js = crate::swift::swap_backticks_for_triple_quotes(js);
        // Join multi-line expressions (open-paren continuation) into single lines
        // so that per-line rewrites can see the full call.
        let js = join_continuation_lines(&js);

        let mut out: Vec<String> = Vec::with_capacity(js.lines().count());
        let mut has_fragmentcolor_import = false;
        // Track declared val names to rename duplicates (Kotlin doesn't allow shadowing)
        let mut declared_vals: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        // Track which stub variables we've already emitted (from hidden Rust doc lines).
        let mut emitted_stubs: std::collections::HashSet<String> = std::collections::HashSet::new();
        // Known hidden-variable stubs: variables declared in `# ...` doc lines that are
        // referenced in visible lines. We emit a placeholder declaration on first use.
        let hidden_var_stubs: &[(&str, &str)] = &[
            ("encoded_png_bytes", "val encoded_png_bytes: ByteArray = byteArrayOf()"),
            ("raw_rgba", "val raw_rgba: ByteArray = ByteArray(8 * 8 * 4)"),
        ];

        for raw in js.lines() {
            let line = raw.to_string();
            let trimmed = line.trim_start();

            // Drop JS-only import lines (replaced by the package-level import below)
            if trimmed.starts_with("import ") && trimmed.contains("from \"fragmentcolor\"") {
                if !has_fragmentcolor_import {
                    out.push("import org.fragmentcolor.*".to_string());
                    has_fragmentcolor_import = true;
                }
                continue;
            }

            // HEADLESS guard: lines that use the browser DOM API (document.createElement)
            // have no equivalent on Android. Replace the whole line with a comment so the
            // function still compiles. The `createTarget(canvas)` call on the next line
            // must also be removed — handled by rewrite_line via the createTarget pattern.
            if trimmed.contains("document.createElement") {
                let indent_len = line.len() - trimmed.len();
                let indent = &line[..indent_len];
                out.push(format!("{}// HEADLESS: canvas creation not needed on Android", indent));
                continue;
            }

            // Before rewriting: check if this line references any known hidden-doc variables
            // that were defined in `# ...` Rust lines (invisible to the transpiler). If so,
            // emit a stub declaration before the first reference to each such variable.
            let indent_len = line.len() - trimmed.len();
            let indent = &line[..indent_len];
            for (var_name, stub_decl) in hidden_var_stubs {
                // Only emit a stub if this line uses the var as an identifier and we haven't
                // already declared it (either via a stub or a real val statement).
                let used = {
                    let mut found = false;
                    let mut pos = 0usize;
                    while let Some(p) = trimmed[pos..].find(var_name) {
                        let abs = pos + p;
                        let before_ok = abs == 0 || !trimmed.as_bytes()[abs - 1].is_ascii_alphanumeric() && trimmed.as_bytes()[abs - 1] != b'_';
                        let after_pos = abs + var_name.len();
                        let after_ok = after_pos >= trimmed.len() || !trimmed.as_bytes()[after_pos].is_ascii_alphanumeric() && trimmed.as_bytes()[after_pos] != b'_';
                        if before_ok && after_ok { found = true; break; }
                        pos = abs + 1;
                    }
                    found
                };
                let already_declared = emitted_stubs.contains(*var_name)
                    || declared_vals.contains_key(*var_name);
                if used && !already_declared {
                    out.push(format!("{}{}", indent, stub_decl));
                    emitted_stubs.insert(var_name.to_string());
                }
            }

            let rewritten = rewrite_line(&line);
            // Drop lines that are just `val _ = ...` (Kotlin reserves `_`)
            let trimmed_rewritten = rewritten.trim_start();
            if trimmed_rewritten.starts_with("val _ =") || trimmed_rewritten == "val _" {
                continue;
            }
            // Handle Kotlin destructuring: `val (a, b) = expr` is not supported for
            // non-data-class types. Expand to two separate val declarations.
            let rewritten = if let Some(expanded) = expand_destructuring_val(&rewritten) {
                expanded
            } else {
                rewritten
            };
            // Rename duplicate `val <name>` declarations to avoid conflicting declarations.
            let rewritten = rename_duplicate_val(&rewritten, &mut declared_vals);
            out.push(rewritten);
        }

        out.join("\n")
    }

    fn rewrite_line(src: &str) -> String {
        let mut out = src.to_string();

        if let Some(stripped) = out.trim_end().strip_suffix(';') {
            let leading_len = out.len() - out.trim_start().len();
            let leading = &src[..leading_len];
            out = format!("{}{}", leading, stripped.trim_start());
        }

        out = drop_new_keyword(&out);
        out = replace_leading_keyword(&out, "const ", "val ");
        out = drop_await_prefix(&out);
        out = drop_unwrap(&out);
        out = drop_rust_slice_suffix(&out);
        out = rewrite_array_fill(&out);
        out = swap_single_quoted_strings(&out);
        // Convert [a, b, c] → arrayOf(a, b, c) FIRST so all subsequent
        // API-specific rewrites see a uniform `arrayOf(...)` form.
        // Repeat until stable to handle nested bracket arrays like [[a,b],[c,d]].
        loop {
            let next = rewrite_bracket_array_to_arrayof(&out);
            if next == out { break; }
            out = next;
        }
        // API-specific rewrites (all now see arrayOf(...) for array literals)
        out = rewrite_create_target_canvas(&out);
        out = rewrite_createtexturetarget_size_array(&out);
        out = rewrite_createdepthtexture_size_array(&out);
        out = rewrite_resize_size_array(&out);
        out = rewrite_createstoragetexture_tuple(&out);
        out = rewrite_createtexture_path(&out);
        out = rewrite_createtexture_pixels_tuple(&out);
        out = rewrite_texturemipchain_prepare_tuple(&out);
        out = rewrite_setviewport_tuple(&out);
        out = rewrite_setcomputedispatch_int_to_uint(&out);
        out = rewrite_render_array_to_list(&out);
        out = rewrite_removemeshes_array_to_list(&out);
        out = rewrite_addinstances_array_to_list(&out);
        out = rewrite_addvertices_array_to_list(&out);
        out = rewrite_fromvertices_array_to_list(&out);
        out = rewrite_addvertex_array_to_vertex(&out);
        out = rewrite_vertex_constructor_array(&out);
        out = rewrite_setclearcolor_array(&out);
        out = rewrite_shader_set_array(&out);
        out = rewrite_readtexture_to_getimage(&out);
        out = rewrite_textureformat_camel_to_screaming(&out);
        out = rewrite_instance_new(&out);
        out = rewrite_arrayof_byte_literals(&out);
        out = rewrite_shader_new_to_compose(&out);
        out = rewrite_quad_arrayof_to_listof(&out);
        out = rewrite_texture_region_name(&out);
        out = rewrite_write_region_array_arg(&out);
        out = rewrite_unregister_texture_id(&out);
        out = rewrite_set_instance_count_uint(&out);
        out = rewrite_instance_set_vertex_value(&out);
        out = rewrite_arrayof_float_to_listof(&out);
        out = drop_underscore_val(&out);
        out = rewrite_levels_indexing(&out);

        out
    }

    // Drop `.unwrap()` — Kotlin uses exceptions, not Result types.
    fn drop_unwrap(line: &str) -> String {
        line.replace(".unwrap()", "")
    }

    // Drop Rust slice syntax `[..]` and `.asSlice()` after identifiers.
    fn drop_rust_slice_suffix(line: &str) -> String {
        let mut out = line.to_string();
        while let Some(pos) = out.find("[..]") {
            out = format!("{}{}", &out[..pos], &out[pos + 4..]);
        }
        // `.asSlice()` is a Rust-specific method — drop it entirely
        out = out.replace(".asSlice()", "");
        out
    }

    // `createTarget(canvas)` is web-only (HTMLCanvas). On Android it requires a
    // Surface. The healthcheck examples that create a canvas create a TextureTarget
    // instead (same size as the canvas was intended to be), or skip the call.
    // We replace `renderer.createTarget(canvas)` with a texture-target fallback.
    fn rewrite_create_target_canvas(line: &str) -> String {
        // Match `<var> = <renderer>.createTarget(canvas)` → texture target
        if line.contains("createTarget(canvas)") {
            return line.replace("createTarget(canvas)", "createTextureTarget(800u, 600u)");
        }
        line.to_string()
    }

    // `createTextureTarget(arrayOf(w, h))` → `createTextureTarget(w.toUInt(), h.toUInt())`
    // The uniffi API takes `(width: UInt, height: UInt)` as separate primitive args.
    fn rewrite_createtexturetarget_size_array(line: &str) -> String {
        rewrite_two_arg_size_call(line, "createTextureTarget")
    }

    // `createDepthTexture(arrayOf(w, h))` → `createDepthTexture(w.toUInt(), h.toUInt())`
    fn rewrite_createdepthtexture_size_array(line: &str) -> String {
        rewrite_two_arg_size_call(line, "createDepthTexture")
    }

    // `resize(arrayOf(w, h))` → `resize(w.toUInt(), h.toUInt())`
    fn rewrite_resize_size_array(line: &str) -> String {
        rewrite_two_arg_size_call(line, "resize")
    }

    // Generic helper: `fn([w, h])` or `fn(arrayOf(w, h))` → `fn(w.toUInt(), h.toUInt())`
    fn rewrite_two_arg_size_call(line: &str, fn_name: &str) -> String {
        let needle = format!("{}(", fn_name);
        if let Some(start) = line.find(&needle) {
            let call_start = start + needle.len();
            let chars: Vec<char> = line.chars().collect();
            // Find matching closing paren
            let mut depth = 0i32;
            let mut end = None;
            let mut j = call_start;
            while j < chars.len() {
                match chars[j] {
                    '(' | '[' => depth += 1,
                    ')' => {
                        if depth == 0 { end = Some(j); break; }
                        depth -= 1;
                    }
                    ']' => { depth -= 1; }
                    _ => {}
                }
                j += 1;
            }
            let Some(end) = end else { return line.to_string(); };
            let inner: String = chars[call_start..end].iter().collect();
            let inner_trim = inner.trim();

            // Pattern: `[w, h]` → extract w and h (JS array literal)
            if inner_trim.starts_with('[') && inner_trim.ends_with(']') {
                let args_str = &inner_trim[1..inner_trim.len() - 1];
                let args: Vec<&str> = args_str.splitn(2, ',').collect();
                if args.len() == 2 {
                    let w = to_uint_arg(args[0].trim());
                    let h = to_uint_arg(args[1].trim());
                    let before = &line[..start + needle.len() - 1];
                    let after = &line[end + 1..];
                    return format!("{}({}, {}){}", before, w, h, after);
                }
            }

            // Pattern: `arrayOf(w, h)` → extract w and h
            if inner_trim.starts_with("arrayOf(") && inner_trim.ends_with(')') {
                let args_str = &inner_trim["arrayOf(".len()..inner_trim.len() - 1];
                let args: Vec<&str> = args_str.splitn(2, ',').collect();
                if args.len() == 2 {
                    let w = to_uint_arg(args[0].trim());
                    let h = to_uint_arg(args[1].trim());
                    let before = &line[..start + needle.len() - 1];
                    let after = &line[end + 1..];
                    return format!("{}({}, {}){}", before, w, h, after);
                }
            }
        }
        line.to_string()
    }

    // Convert an integer literal argument to UInt form.
    // `64` → `64u`  (Kotlin UInt literal suffix)
    // `64u` → `64u` (already done)
    // `myVar` → `myVar.toUInt()` (variable)
    fn to_uint_arg(s: &str) -> String {
        let s = s.trim();
        if s.ends_with('u') {
            return s.to_string(); // already UInt
        }
        if s.chars().all(|c| c.is_ascii_digit() || c == '_') {
            return format!("{}u", s);
        }
        // Variable or expression — coerce
        format!("{}.toUInt()", s)
    }

    // `createStorageTexture((arrayOf(w, h), TextureFormat.X))` →
    // `createStorageTexture(Size(width=w.toUInt(), height=h.toUInt(), depth=null), TextureFormat.X, null, null)`
    //
    // Also handles the three-arg form:
    // `createStorageTexture((arrayOf(w, h), TextureFormat.X, pixels))` →
    // `createStorageTexture(Size(...), TextureFormat.X, pixels.toByteArray(), null)`
    fn rewrite_createstoragetexture_tuple(line: &str) -> String {
        let needle = "createStorageTexture(";
        let Some(start) = line.find(needle) else { return line.to_string(); };
        let call_start = start + needle.len();
        let chars: Vec<char> = line.chars().collect();

        // The next char should be `(` (a JS-style tuple argument)
        if call_start >= chars.len() || chars[call_start] != '(' {
            return line.to_string();
        }

        // Walk the outer paren from call_start to find the matching `)` of the tuple
        let mut depth = 0i32;
        let mut tuple_end = None;
        let mut j = call_start;
        while j < chars.len() {
            match chars[j] {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 { tuple_end = Some(j); break; }
                }
                _ => {}
            }
            j += 1;
        }

        let Some(tuple_end) = tuple_end else { return line.to_string(); };
        if tuple_end <= call_start { return line.to_string(); }

        // The content between the outer parens
        let tuple_inner: String = chars[call_start + 1..tuple_end].iter().collect();
        // There may be a trailing `)` after tuple_end that closes the createStorageTexture call
        // We need to find the closing `)` of createStorageTexture()
        let after_tuple: String = chars[tuple_end + 1..].iter().collect();
        let after_tuple = after_tuple.trim_start_matches(')');

        // Parse the comma-separated parts of the tuple
        // We need to split respecting nested parens
        let parts = split_args(&tuple_inner);
        let before = &line[..start + needle.len() - 1]; // everything up to but not including the `(`

        match parts.as_slice() {
            [size_expr, format_expr] => {
                let size = parse_size_to_kotlin(size_expr.trim());
                let fmt = rewrite_textureformat_camel_to_screaming(format_expr.trim());
                format!("{}({}, {}, null, null){}", before, size, fmt, after_tuple)
            }
            [size_expr, format_expr, data_expr] => {
                let size = parse_size_to_kotlin(size_expr.trim());
                let fmt = rewrite_textureformat_camel_to_screaming(format_expr.trim());
                let data = convert_array_to_bytearray(data_expr.trim());
                format!("{}({}, {}, {}, null){}", before, size, fmt, data, after_tuple)
            }
            _ => line.to_string(),
        }
    }

    // Convert `[w, h]` or `arrayOf(w, h)` → `Size(width=Xu, height=Yu, depth=null)`
    fn parse_size_to_kotlin(s: &str) -> String {
        // Handle `[w, h]` form (JS array literal, before bracket rewrite)
        if s.starts_with('[') && s.ends_with(']') {
            let inner = &s[1..s.len() - 1];
            let args: Vec<&str> = inner.splitn(2, ',').collect();
            if args.len() == 2 {
                let w = to_uint_arg(args[0].trim());
                let h = to_uint_arg(args[1].trim());
                return format!("Size(width={}, height={}, depth=null)", w, h);
            }
        }
        // Handle `arrayOf(w, h)` form (after bracket rewrite)
        if s.starts_with("arrayOf(") && s.ends_with(')') {
            let inner = &s["arrayOf(".len()..s.len() - 1];
            let args: Vec<&str> = inner.splitn(2, ',').collect();
            if args.len() == 2 {
                let w = to_uint_arg(args[0].trim());
                let h = to_uint_arg(args[1].trim());
                return format!("Size(width={}, height={}, depth=null)", w, h);
            }
        }
        // Fallback: pass through (may be already correct)
        s.to_string()
    }

    // Convert integer/double array to ByteArray for data param of createStorageTexture
    fn convert_array_to_bytearray(s: &str) -> String {
        // `Array(N) { 0 }` → keep as-is but note that write() takes ByteArray
        // For now pass through — actual type conversion is the caller's concern
        s.to_string()
    }

    // `createTexture(image[..])` → `createTexture(TextureInputMobile.Path(image), null)`
    // The `[..]` was already stripped by `drop_rust_slice_suffix`, so the
    // variable name is already clean.
    fn rewrite_createtexture_path(line: &str) -> String {
        // Detect `createTexture(<string_var>)` where the arg is a val holding a path string,
        // not already a TextureInputMobile call.
        let needle = "createTexture(";
        let Some(start) = line.find(needle) else { return line.to_string(); };
        let call_start = start + needle.len();
        let chars: Vec<char> = line.chars().collect();

        if call_start >= chars.len() {
            return line.to_string();
        }

        // Find matching closing paren
        let mut depth = 0i32;
        let mut end = call_start;
        let mut j = call_start;
        while j < chars.len() {
            match chars[j] {
                '(' => depth += 1,
                ')' => {
                    if depth == 0 { end = j; break; }
                    depth -= 1;
                }
                _ => {}
            }
            j += 1;
        }

        let inner: String = chars[call_start..end].iter().collect();
        let inner_trim = inner.trim();

        // If already starts with TextureInputMobile, skip
        if inner_trim.starts_with("TextureInputMobile") {
            return line.to_string();
        }
        // If it's a tuple `(...)`, skip — handled by rewrite_createtexture_pixels_tuple
        if inner_trim.starts_with('(') {
            return line.to_string();
        }
        // If the arg is a simple identifier or string (path case)
        // Check it doesn't look like a bytes argument
        if !inner_trim.contains("arrayOf") && !inner_trim.contains("Array(") && !inner_trim.contains(',') {
            let before = &line[..start + needle.len() - 1];
            let after = &line[end + 1..];
            // Detect TextureMipChain variables by naming convention
            let is_mip_chain = inner_trim.contains("chain") || inner_trim.contains("mip");
            let variant = if is_mip_chain { "Prepared" } else { "Path" };
            return format!("{}(TextureInputMobile.{}({}), null){}", before, variant, inner_trim, after);
        }

        line.to_string()
    }

    // `createTexture((pixels, arrayOf(w, h)))` → `createTexture(TextureInputMobile.Bytes(pixels.toByteArray()), null)`
    // where the tuple form signals raw pixels + size.
    fn rewrite_createtexture_pixels_tuple(line: &str) -> String {
        let needle = "createTexture(";
        let Some(start) = line.find(needle) else { return line.to_string(); };
        let call_start = start + needle.len();
        let chars: Vec<char> = line.chars().collect();

        if call_start >= chars.len() || chars[call_start] != '(' {
            return line.to_string();
        }

        // Walk the outer tuple paren
        let mut depth = 0i32;
        let mut tuple_end = None;
        let mut j = call_start;
        while j < chars.len() {
            match chars[j] {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 { tuple_end = Some(j); break; }
                }
                _ => {}
            }
            j += 1;
        }

        let Some(tuple_end) = tuple_end else { return line.to_string(); };
        if tuple_end <= call_start { return line.to_string(); }

        let tuple_inner: String = chars[call_start + 1..tuple_end].iter().collect();
        let after_tuple: String = chars[tuple_end + 1..].iter().collect();
        let after_tuple = after_tuple.trim_start_matches(')');

        let parts = split_args(&tuple_inner);
        let before = &line[..start + needle.len() - 1];

        match parts.as_slice() {
            [pixels_expr, _size_expr] => {
                // (pixels, [w, h]) — raw bytes with size
                let px = pixels_expr.trim();
                // We use TextureInputMobile.Bytes for raw pixel data
                format!("{}(TextureInputMobile.Bytes({}.let {{ ba -> ByteArray(ba.size) {{ i -> ba[i].toInt().and(0xFF).toByte() }} }}), null){}", before, px, after_tuple)
            }
            _ => line.to_string(),
        }
    }

    // `TextureMipChain.prepare((bytes, format))` or
    // `TextureMipChain.prepare((bytes, format, arrayOf(w,h)))` →
    // `TextureMipChain.prepare(bytes, format, null)` or
    // `TextureMipChain.prepare(bytes, format, Size(...))`
    fn rewrite_texturemipchain_prepare_tuple(line: &str) -> String {
        let needle = "TextureMipChain.prepare(";
        let Some(start) = line.find(needle) else { return line.to_string(); };
        let call_start = start + needle.len();
        let chars: Vec<char> = line.chars().collect();

        if call_start >= chars.len() || chars[call_start] != '(' {
            return line.to_string();
        }

        // Walk the outer tuple paren
        let mut depth = 0i32;
        let mut tuple_end = None;
        let mut j = call_start;
        while j < chars.len() {
            match chars[j] {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 { tuple_end = Some(j); break; }
                }
                _ => {}
            }
            j += 1;
        }

        let Some(tuple_end) = tuple_end else { return line.to_string(); };
        if tuple_end <= call_start { return line.to_string(); }

        let tuple_inner: String = chars[call_start + 1..tuple_end].iter().collect();
        let after_tuple: String = chars[tuple_end + 1..].iter().collect();
        let after_tuple = after_tuple.trim_start_matches(')');

        let parts = split_args(&tuple_inner);
        let before = &line[..start + needle.len() - 1];

        match parts.as_slice() {
            [bytes_expr, format_expr] => {
                let fmt = rewrite_textureformat_camel_to_screaming(format_expr.trim());
                // Bytes could be raw variable or arrayOf — wrap in ByteArray if needed
                let b = bytes_to_bytearray(bytes_expr.trim());
                format!("{}({}, {}, null){}", before, b, fmt, after_tuple)
            }
            [bytes_expr, format_expr, size_expr] => {
                let fmt = rewrite_textureformat_camel_to_screaming(format_expr.trim());
                let b = bytes_to_bytearray(bytes_expr.trim());
                let size = parse_size_to_kotlin(size_expr.trim());
                format!("{}({}, {}, {}){}", before, b, fmt, size, after_tuple)
            }
            _ => line.to_string(),
        }
    }

    // Convert a bytes expression to a ByteArray form Kotlin can use.
    // `Array(N) { 0 }` → convert to ByteArray
    // plain variable → `.let { ba -> ByteArray(ba.size) { i -> ba[i].toByte() } }` if it looks like an Int array
    fn bytes_to_bytearray(s: &str) -> String {
        if s.starts_with("Array(") {
            // `Array(N) { 0 }` — convert to ByteArray
            return format!("ByteArray({}.size) {{ i -> {}[i].toByte() }}", s, s);
        }
        // Unknown — pass through (may already be ByteArray from override file)
        s.to_string()
    }

    // `pass.setViewport(arrayOf((0, 0), (32, 32)))` →
    // `pass.setViewport(ScreenRegion(minX=0u, minY=0u, maxX=32u, maxY=32u))`
    //
    // The JS source `[region]` = `[[x1,y1],[x2,y2]]` representing top-left/bottom-right.
    // ScreenRegion takes (minX, minY, maxX, maxY).
    fn rewrite_setviewport_tuple(line: &str) -> String {
        let needle = "setViewport(";
        let Some(start) = line.find(needle) else { return line.to_string(); };
        let call_start = start + needle.len();
        let chars: Vec<char> = line.chars().collect();

        let mut depth = 0i32;
        let mut end = call_start;
        let mut j = call_start;
        while j < chars.len() {
            match chars[j] {
                '(' => depth += 1,
                ')' => {
                    if depth == 0 { end = j; break; }
                    depth -= 1;
                }
                _ => {}
            }
            j += 1;
        }

        let inner: String = chars[call_start..end].iter().collect();
        let inner_trim = inner.trim();

        // Already a ScreenRegion?
        if inner_trim.starts_with("ScreenRegion") {
            return line.to_string();
        }

        // `arrayOf((x1, y1), (x2, y2))` or plain `arrayOf(0, 0, w, h)` form
        // Also handle `arrayOf((x, y), (x2, y2))` — the tuple-of-tuples from JS conversion
        if inner_trim.starts_with("arrayOf(") && inner_trim.ends_with(')') {
            let args_str = &inner_trim["arrayOf(".len()..inner_trim.len() - 1];
            // Try to extract 4 integers from potentially nested parens
            let nums = extract_all_integers(args_str);
            if nums.len() == 4 {
                let before = &line[..start + needle.len() - 1];
                let after = &line[end + 1..];
                return format!(
                    "{}(ScreenRegion(minX={}u, minY={}u, maxX={}u, maxY={}u)){}",
                    before, nums[0], nums[1], nums[2], nums[3], after
                );
            }
        }

        line.to_string()
    }

    // Extract all integer literals from a string (for parsing viewport tuples)
    fn extract_all_integers(s: &str) -> Vec<String> {
        let mut result = Vec::new();
        let mut current = String::new();
        for c in s.chars() {
            if c.is_ascii_digit() {
                current.push(c);
            } else {
                if !current.is_empty() {
                    result.push(current.clone());
                    current.clear();
                }
            }
        }
        if !current.is_empty() {
            result.push(current);
        }
        result
    }

    // `renderer.render(arrayOf(pass, pass2), target)` →
    // `renderer.render(listOf(pass, pass2), target)`
    fn rewrite_render_array_to_list(line: &str) -> String {
        // Replace `render(arrayOf(` → `render(listOf(` only when the arg is a collection of Pass objects
        // We detect this by looking for `arrayOf(` immediately after `render(`
        let needle = ".render(arrayOf(";
        if line.contains(needle) {
            return line.replace(needle, ".render(listOf(");
        }
        line.to_string()
    }

    // `pass.setComputeDispatch(64, 64, 1)` → `pass.setComputeDispatch(64u, 64u, 1u)`
    //
    // The uniffi-generated member method takes `(UInt, UInt, UInt)` and Kotlin
    // member methods take precedence over the Int-arg extension in
    // PassExtensions.kt. Signed→unsigned conversion of integer constants is
    // disallowed, so we suffix each literal with `u` to make them UInt.
    fn rewrite_setcomputedispatch_int_to_uint(line: &str) -> String {
        let needle = ".setComputeDispatch(";
        let Some(start) = line.find(needle) else {
            return line.to_string();
        };
        let call_start = start + needle.len();
        let chars: Vec<char> = line.chars().collect();
        let mut depth = 0i32;
        let mut end = call_start;
        let mut j = call_start;
        while j < chars.len() {
            match chars[j] {
                '(' => depth += 1,
                ')' => {
                    if depth == 0 {
                        end = j;
                        break;
                    }
                    depth -= 1;
                }
                _ => {}
            }
            j += 1;
        }
        let inner: String = chars[call_start..end].iter().collect();
        // Suffix each comma-separated bare integer with `u`
        let parts: Vec<String> = inner
            .split(',')
            .map(|p| {
                let trimmed = p.trim();
                if trimmed.parse::<u64>().is_ok() {
                    format!("{}u", trimmed)
                } else {
                    p.to_string()
                }
            })
            .collect();
        let new_inner = parts.join(",");
        let before = &line[..call_start];
        let after = &line[end..];
        format!("{}{}{}", before, new_inner, after)
    }

    // `shader.removeMeshes(arrayOf(m1, m2))` → `shader.removeMeshes(listOf(m1, m2))`
    fn rewrite_removemeshes_array_to_list(line: &str) -> String {
        let needle = "removeMeshes(arrayOf(";
        if line.contains(needle) {
            return line.replace(needle, "removeMeshes(listOf(");
        }
        line.to_string()
    }

    // `m.addInstances([...])` → `m.addInstances(listOf(...))`
    // The `[...]` is converted to `arrayOf(...)` by rewrite_bracket_array_to_arrayof.
    fn rewrite_addinstances_array_to_list(line: &str) -> String {
        let needle = "addInstances(arrayOf(";
        if line.contains(needle) {
            return line.replace(needle, "addInstances(listOf(");
        }
        line.to_string()
    }

    // `m.addVertices(listOf(arrayOf(x, y), ...))` → `m.addVertices(listOf(Vertex(listOf(xf, yf)), ...))`
    // Also converts `addVertices(arrayOf(` → `addVertices(listOf(` first.
    fn rewrite_addvertices_array_to_list(line: &str) -> String {
        let line = if line.contains("addVertices(arrayOf(") {
            line.replace("addVertices(arrayOf(", "addVertices(listOf(")
        } else {
            line.to_string()
        };
        // Now convert inner arrayOf elements to Vertex(listOf(...))
        convert_inner_array_elements_to_vertex(&line, "addVertices(listOf(")
    }

    // `Mesh.fromVertices(listOf(arrayOf(x, y), ...))` → `Mesh.fromVertices(listOf(Vertex(listOf(xf, yf)), ...))`
    fn rewrite_fromvertices_array_to_list(line: &str) -> String {
        let line = if line.contains("fromVertices(arrayOf(") {
            line.replace("fromVertices(arrayOf(", "fromVertices(listOf(")
        } else {
            line.to_string()
        };
        convert_inner_array_elements_to_vertex(&line, "fromVertices(listOf(")
    }

    // Convert `arrayOf(x, y, z)` elements inside a `listOf(...)` call (at fn_prefix) to
    // `Vertex(listOf(xf, yf, zf))`.
    fn convert_inner_array_elements_to_vertex(line: &str, fn_prefix: &str) -> String {
        let Some(start) = line.find(fn_prefix) else { return line.to_string(); };
        let list_start = start + fn_prefix.len();
        let chars: Vec<char> = line.chars().collect();

        // Find the matching close paren of listOf(
        let mut depth = 0i32;
        let mut list_end = None;
        let mut j = list_start;
        while j < chars.len() {
            match chars[j] {
                '(' => depth += 1,
                ')' => {
                    if depth == 0 { list_end = Some(j); break; }
                    depth -= 1;
                }
                _ => {}
            }
            j += 1;
        }
        let Some(list_end) = list_end else { return line.to_string(); };

        let list_inner: String = chars[list_start..list_end].iter().collect();
        // Split into elements
        let elements = split_args(&list_inner);

        // Convert each element that is `arrayOf(...)` to `Vertex(listOf(...))`
        let converted: Vec<String> = elements.iter().map(|el| {
            let el = el.trim();
            if el.starts_with("arrayOf(") && el.ends_with(')') {
                let floats_str = &el["arrayOf(".len()..el.len() - 1];
                let list_floats = convert_to_float_list(floats_str);
                format!("Vertex(listOf({}))", list_floats)
            } else {
                el.to_string()
            }
        }).collect();

        let before = &line[..start + fn_prefix.len() - 1]; // up to but not incl '('
        let after: String = chars[list_end + 1..].iter().collect();
        format!("{}({}){}", before, converted.join(", "), after)
    }

    // `mesh.addVertex(arrayOf(0.0, 0.0, 0.0))` → `mesh.addVertex(Vertex(listOf(0.0f, 0.0f, 0.0f)))`
    // `mesh.addVertex(Vertex.new(arrayOf(...)))` → same
    fn rewrite_addvertex_array_to_vertex(line: &str) -> String {
        let needle = "addVertex(";
        let Some(start) = line.find(needle) else { return line.to_string(); };
        let call_start = start + needle.len();
        let chars: Vec<char> = line.chars().collect();

        let mut depth = 0i32;
        let mut end = call_start;
        let mut j = call_start;
        while j < chars.len() {
            match chars[j] {
                '(' => depth += 1,
                ')' => {
                    if depth == 0 { end = j; break; }
                    depth -= 1;
                }
                _ => {}
            }
            j += 1;
        }

        let inner: String = chars[call_start..end].iter().collect();
        let inner_trim = inner.trim();

        // Already a Vertex? Skip
        if inner_trim.starts_with("Vertex(") || inner_trim.starts_with("Vertex.new(") {
            // strip `Vertex.new(arrayOf(...))` → `Vertex(listOf(...))`
            if inner_trim.starts_with("Vertex.new(arrayOf(") && inner_trim.ends_with("))") {
                let floats_str = &inner_trim["Vertex.new(arrayOf(".len()..inner_trim.len() - 2];
                let list_floats = convert_to_float_list(floats_str);
                let before = &line[..start + needle.len() - 1];
                let after = &line[end + 1..];
                return format!("{}(Vertex(listOf({}))){}",  before, list_floats, after);
            }
            // Already `Vertex(...)` — pass through
            return line.to_string();
        }

        // `arrayOf(0.0, 0.0)` → `Vertex(listOf(0.0f, 0.0f))`
        if inner_trim.starts_with("arrayOf(") && inner_trim.ends_with(')') {
            let floats_str = &inner_trim["arrayOf(".len()..inner_trim.len() - 1];
            let list_floats = convert_to_float_list(floats_str);
            let before = &line[..start + needle.len() - 1];
            let after = &line[end + 1..];
            // before = "mesh.addVertex" (no trailing `(`), format adds `(Vertex(listOf(X)))`
            // Pattern: {before}(Vertex(listOf({floats}))){after}  — 3 closing parens
            return format!("{}(Vertex(listOf({}))){}",  before, list_floats, after);
        }

        line.to_string()
    }

    // `Vertex(arrayOf(...))` → `Vertex(listOf(...f, ...))`
    fn rewrite_vertex_constructor_array(line: &str) -> String {
        let needle = "Vertex(arrayOf(";
        if let Some(start) = line.find(needle) {
            let inner_start = start + needle.len();
            let chars: Vec<char> = line.chars().collect();
            // Find the matching `)` for the arrayOf
            let mut depth = 0i32;
            let mut array_end = inner_start;
            let mut j = inner_start;
            while j < chars.len() {
                match chars[j] {
                    '(' => depth += 1,
                    ')' => {
                        if depth == 0 { array_end = j; break; }
                        depth -= 1;
                    }
                    _ => {}
                }
                j += 1;
            }
            let floats_str: String = chars[inner_start..array_end].iter().collect();
            // Check that after `arrayOf(...)` there's a `)` closing `Vertex(`
            let after_array: String = chars[array_end + 1..].iter().collect();
            if let Some(rest) = after_array.strip_prefix(')') {
                let before = &line[..start];
                let list_floats = convert_to_float_list(&floats_str);
                return format!("{}Vertex(listOf({})){}",  before, list_floats, rest);
            }
        }
        line.to_string()
    }

    // `pass.setClearColor(arrayOf(r, g, b, a))` → `pass.setClearColor(listOf(rf, gf, bf, af))`
    fn rewrite_setclearcolor_array(line: &str) -> String {
        let needle = "setClearColor(arrayOf(";
        if let Some(start) = line.find(needle) {
            let inner_start = start + needle.len();
            let chars: Vec<char> = line.chars().collect();
            let mut depth = 0i32;
            let mut array_end = inner_start;
            let mut j = inner_start;
            while j < chars.len() {
                match chars[j] {
                    '(' => depth += 1,
                    ')' => {
                        if depth == 0 { array_end = j; break; }
                        depth -= 1;
                    }
                    _ => {}
                }
                j += 1;
            }
            let floats_str: String = chars[inner_start..array_end].iter().collect();
            // After the arrayOf(...) there's a `)` closing setClearColor
            let after_array: String = chars[array_end + 1..].iter().collect();
            if let Some(rest) = after_array.strip_prefix(')') {
                let before = &line[..start];
                let list_floats = convert_to_float_list(&floats_str);
                return format!("{}setClearColor(listOf({})){}",  before, list_floats, rest);
            }
        }
        line.to_string()
    }

    // `shader.set("key", arrayOf(x, y))` → `shader.set("key", floatArrayOf(xf, yf))`
    // ShaderExtensions.kt has `fun Shader.set(key: String, value: FloatArray)`.
    fn rewrite_shader_set_array(line: &str) -> String {
        // Match `.set(<key>, arrayOf(...))`
        let needle = ", arrayOf(";
        if line.contains(".set(") && line.contains(needle) {
            if let Some(array_start) = line.find(needle) {
                let inner_start = array_start + needle.len();
                let chars: Vec<char> = line.chars().collect();
                let mut depth = 0i32;
                let mut array_end = inner_start;
                let mut j = inner_start;
                while j < chars.len() {
                    match chars[j] {
                        '(' => depth += 1,
                        ')' => {
                            if depth == 0 { array_end = j; break; }
                            depth -= 1;
                        }
                        _ => {}
                    }
                    j += 1;
                }
                let floats_str: String = chars[inner_start..array_end].iter().collect();
                let after_array: String = chars[array_end + 1..].iter().collect();
                // Check this closes the set() call
                if let Some(rest) = after_array.strip_prefix(')') {
                    let before = &line[..array_start];
                    let float_args = convert_to_float_args(&floats_str);
                    return format!("{}, floatArrayOf({})){}",  before, float_args, rest);
                }
            }
        }
        line.to_string()
    }

    // `renderer.readTexture(texture.id())` is now a real uniffi mobile binding —
    // no rewrite needed. The old `readTextureAsync` no longer exists.
    fn rewrite_readtexture_to_getimage(line: &str) -> String {
        line.to_string()
    }

    // Replace `<receiver>.method(arg)` with `replacement` where receiver is any identifier chain.
    fn replace_method_call_with_expr(line: &str, method: &str, arg: &str, replacement: &str) -> String {
        let needle = format!(".{}({})", method, arg);
        if let Some(pos) = line.find(&needle) {
            // Walk back from pos to find the start of the receiver identifier chain
            let before = &line[..pos];
            let receiver_start = before
                .rfind(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '.')
                .map(|p| p + 1)
                .unwrap_or(0);
            let prefix = &line[..receiver_start];
            let after = &line[pos + needle.len()..];
            format!("{}{}{}", prefix, replacement, after)
        } else {
            line.to_string()
        }
    }

    // Rewrite TextureFormat camelCase to SCREAMING_SNAKE_CASE for Kotlin enum.
    // `TextureFormat.Rgba8UnormSrgb` → `TextureFormat.RGBA8_UNORM_SRGB`
    // `TextureFormat.Rgba` → `TextureFormat.RGBA` (Kotlin enum)
    fn rewrite_textureformat_camel_to_screaming(line: &str) -> String {
        let needle = "TextureFormat.";
        let mut result = String::with_capacity(line.len());
        let mut remaining = line;
        while let Some(start) = remaining.find(needle) {
            result.push_str(&remaining[..start + needle.len()]);
            let rest = &remaining[start + needle.len()..];
            let end = rest.find(|c: char| !c.is_ascii_alphanumeric() && c != '_').unwrap_or(rest.len());
            let variant = &rest[..end];
            let screaming = camel_to_screaming_snake(variant);
            result.push_str(&screaming);
            remaining = &rest[end..];
        }
        result.push_str(remaining);
        result
    }

    // Convert CamelCase to SCREAMING_SNAKE_CASE.
    // `Rgba8UnormSrgb` → `RGBA8_UNORM_SRGB`
    // `Rgba` → `RGBA` (but actually in the enum it's just `RGBA`)
    fn camel_to_screaming_snake(s: &str) -> String {
        // If already all-caps or all-caps + digits/underscore, return as-is
        if s.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_') {
            return s.to_string();
        }
        let mut out = String::with_capacity(s.len() * 2);
        let chars: Vec<char> = s.chars().collect();
        for (i, &c) in chars.iter().enumerate() {
            if c.is_ascii_uppercase() && i > 0 {
                // Insert underscore before uppercase letter unless previous was also uppercase
                // or previous was a digit (handles cases like `RGBA8` → don't add _ after digit)
                let prev = chars[i - 1];
                if !prev.is_ascii_uppercase() && !prev.is_ascii_digit() {
                    out.push('_');
                } else if i + 1 < chars.len() && chars[i + 1].is_ascii_lowercase() {
                    // e.g. `RGBa` — uppercase followed by lowercase in sequence
                    out.push('_');
                }
            } else if c.is_ascii_digit() && i > 0 && chars[i - 1].is_ascii_alphabetic() {
                // Don't insert underscore before digit — `Rgba8` → `RGBA8` not `RGBA_8`
            }
            out.push(c.to_ascii_uppercase());
        }
        out
    }

    // `Instance.new()` → `Instance()` (drop `.new`)
    fn rewrite_instance_new(line: &str) -> String {
        line.replace("Instance.new()", "Instance()")
    }

    // Convert comma-separated float values to Kotlin `f`-suffixed floats
    fn convert_to_float_list(s: &str) -> String {
        s.split(',')
            .map(|v| {
                let v = v.trim();
                to_float_literal(v)
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    // Convert comma-separated float values to Kotlin floatArrayOf-compatible args
    fn convert_to_float_args(s: &str) -> String {
        convert_to_float_list(s)
    }

    // Convert a double literal `0.0` or `800.0` to a Kotlin float literal `0.0f`
    fn to_float_literal(s: &str) -> String {
        let s = s.trim();
        if s.ends_with('f') || s.ends_with('F') {
            return s.to_string();
        }
        if s.contains('.') {
            return format!("{}f", s);
        }
        // Integer — add `.0f`
        if s.chars().all(|c| c.is_ascii_digit() || c == '-') {
            return format!("{}.0f", s);
        }
        s.to_string()
    }

    // Split a string by commas at depth 0 (respecting nested parens/brackets)
    fn split_args(s: &str) -> Vec<String> {
        let mut parts: Vec<String> = Vec::new();
        let mut current = String::new();
        let mut depth = 0i32;
        for c in s.chars() {
            match c {
                '(' | '[' => { depth += 1; current.push(c); }
                ')' | ']' => { depth -= 1; current.push(c); }
                ',' if depth == 0 => {
                    parts.push(current.trim().to_string());
                    current = String::new();
                }
                _ => current.push(c),
            }
        }
        let last = current.trim().to_string();
        if !last.is_empty() {
            parts.push(last);
        }
        parts
    }

    /// `[a, b, c]` (JS / Swift array literal) → `arrayOf(a, b, c)` for
    /// Kotlin, since Kotlin only allows `[...]` syntax in annotation
    /// arguments. Bracket-balanced and string-aware so WGSL inside
    /// triple-quoted strings is left alone.
    fn rewrite_bracket_array_to_arrayof(line: &str) -> String {
        let chars: Vec<char> = line.chars().collect();
        let mut out = String::with_capacity(line.len());
        let mut i = 0usize;
        let mut in_dq = false;
        let mut in_tq = false;
        while i < chars.len() {
            // Triple-quote toggle
            if i + 2 < chars.len()
                && chars[i] == '"'
                && chars[i + 1] == '"'
                && chars[i + 2] == '"'
            {
                in_tq = !in_tq;
                out.push_str("\"\"\"");
                i += 3;
                continue;
            }
            if !in_tq && chars[i] == '"' {
                in_dq = !in_dq;
                out.push('"');
                i += 1;
                continue;
            }
            // Skip brackets in annotation positions: `@Foo([...])` is fine.
            // Cheap check: if `[` is preceded by `(` it might still be an
            // expression argument, so we always rewrite outside strings.
            if !in_dq && !in_tq && chars[i] == '[' {
                // Walk to matching `]`.
                let mut depth = 0i32;
                let mut close_pos: Option<usize> = None;
                let mut k = i + 1;
                while k < chars.len() {
                    match chars[k] {
                        '[' => depth += 1,
                        ']' => {
                            if depth == 0 {
                                close_pos = Some(k);
                                break;
                            }
                            depth -= 1;
                        }
                        _ => {}
                    }
                    k += 1;
                }
                if let Some(close) = close_pos {
                    let inner: String = chars[i + 1..close].iter().collect();
                    let inner_trim = inner.trim();
                    // Skip empty `[]` (no items to wrap) and indexer
                    // patterns like `arr[0]` (single ident/expr without
                    // commas — those are subscripts, not collection
                    // literals).
                    if !inner_trim.is_empty() && inner_trim.contains(',') {
                        let prev = if i == 0 { ' ' } else { chars[i - 1] };
                        let prev_is_indexer =
                            prev.is_ascii_alphanumeric() || prev == '_' || prev == ')' || prev == ']';
                        if !prev_is_indexer {
                            out.push_str("arrayOf(");
                            out.push_str(inner_trim);
                            out.push(')');
                            i = close + 1;
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

    /// `Array(N).fill(expr)` → `ByteArray(N)` (Kotlin byte array for texture pixel data).
    /// Bracket-balanced; converts all pixel-buffer array-fill patterns to ByteArray.
    fn rewrite_array_fill(line: &str) -> String {
        let needle = "Array(";
        let mut out = String::with_capacity(line.len());
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0usize;
        while i < chars.len() {
            let rem: String = chars[i..].iter().collect();
            if rem.starts_with(needle) {
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
                            // Always produce ByteArray — all pixel-buffer fills are byte arrays
                            out.push_str(&format!("ByteArray({})", count.trim()));
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

    /// Single-quoted JS string → Kotlin double-quoted. Skips content
    /// inside existing `"..."` / `"""..."""` strings.
    fn swap_single_quoted_strings(line: &str) -> String {
        let chars: Vec<char> = line.chars().collect();
        let mut out = String::with_capacity(line.len());
        let mut i = 0usize;
        let mut in_dq = false;
        let mut in_tq = false;
        while i < chars.len() {
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

    fn drop_await_prefix(line: &str) -> String {
        let chars: Vec<char> = line.chars().collect();
        let mut out = String::with_capacity(line.len());
        let mut i = 0usize;
        while i < chars.len() {
            if chars[i..].starts_with(&['a', 'w', 'a', 'i', 't', ' ']) {
                let left_ok = i == 0 || !is_ident_char(chars[i - 1]);
                let right_ok =
                    i + 6 < chars.len() && (chars[i + 6].is_alphabetic() || chars[i + 6] == '_');
                if left_ok && right_ok {
                    i += 6; // skip "await "
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
                    i += 4;
                    continue;
                }
            }
            out.push(chars[i]);
            i += 1;
        }
        out
    }

    /// Rename duplicate `val <name>` declarations by appending `_N`.
    /// Mutates `declared` to track seen names.
    fn rename_duplicate_val(line: &str, declared: &mut std::collections::HashMap<String, u32>) -> String {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("val ") { return line.to_string(); }
        // Extract the val name (before `=` or `:`)
        let rest = &trimmed["val ".len()..];
        // Find end of name (alphanumeric / underscore)
        let name_end = rest.find(|c: char| !c.is_ascii_alphanumeric() && c != '_').unwrap_or(rest.len());
        if name_end == 0 { return line.to_string(); }
        let name = &rest[..name_end];
        // Skip `_` or names that start with `(`
        if name == "_" || name.starts_with('(') { return line.to_string(); }

        let count = declared.entry(name.to_string()).or_insert(0);
        *count += 1;
        if *count == 1 {
            // First occurrence — no rename needed
            return line.to_string();
        }
        // Rename: replace the first occurrence of `val <name>` with `val <name>_<N>`
        let indent_len = line.len() - trimmed.len();
        let indent = &line[..indent_len];
        let new_name = format!("{}{}", name, count);
        let renamed_rest = &trimmed["val ".len() + name_end..];
        format!("{}val {}{}", indent, new_name, renamed_rest)
    }

    fn is_ident_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }

    /// Join continuation lines: when a line ends with more open parens than close parens
    /// (outside strings), the next line is a continuation. Join them with a space so that
    /// per-line rewrites can see the complete expression.
    ///
    /// We skip joining inside triple-quoted strings (WGSL shader code etc.).
    fn join_continuation_lines(src: &str) -> String {
        let mut result_lines: Vec<String> = Vec::new();
        let mut pending: Option<String> = None;
        let mut in_tq = false; // inside triple-quoted string

        for raw in src.lines() {
            // Track triple-quote state
            let mut count_tq = 0usize;
            let chars: Vec<char> = raw.chars().collect();
            let mut i = 0usize;
            while i + 2 < chars.len() {
                if chars[i] == '"' && chars[i+1] == '"' && chars[i+2] == '"' {
                    count_tq += 1;
                    i += 3;
                } else {
                    i += 1;
                }
            }
            if count_tq % 2 == 1 {
                in_tq = !in_tq;
            }

            if in_tq {
                // Inside triple-quoted string — don't join
                if let Some(ref mut p) = pending {
                    p.push('\n');
                    p.push_str(raw);
                } else {
                    result_lines.push(raw.to_string());
                }
                continue;
            }

            // Count paren depth of current line (outside strings)
            let depth = paren_depth_of_line(raw);

            if let Some(ref mut p) = pending {
                // Append continuation to pending, stripping trailing // comments first
                let trimmed_continuation = strip_line_comment(raw.trim());
                p.push(' ');
                p.push_str(trimmed_continuation.trim_end());
                // Recalculate depth of the accumulated line
                let acc_depth = paren_depth_of_line(p);
                if acc_depth <= 0 {
                    result_lines.push(pending.take().unwrap());
                }
            } else if depth > 0 {
                // Start of a multi-line expression; strip trailing comment
                let line_no_comment = strip_line_comment(raw).to_string();
                pending = Some(line_no_comment);
            } else {
                result_lines.push(raw.to_string());
            }
        }
        if let Some(p) = pending {
            result_lines.push(p);
        }
        result_lines.join("\n")
    }

    /// Strip a trailing `//` line comment from a Kotlin/JS line (outside strings).
    fn strip_line_comment(line: &str) -> &str {
        let chars: Vec<char> = line.chars().collect();
        let mut in_str = false;
        let mut in_tq = false;
        let mut i = 0usize;
        while i < chars.len() {
            if !in_str && !in_tq && i + 2 < chars.len()
                && chars[i] == '"' && chars[i+1] == '"' && chars[i+2] == '"' {
                in_tq = !in_tq;
                i += 3; continue;
            }
            if !in_tq && chars[i] == '"' { in_str = !in_str; }
            if !in_str && !in_tq && i + 1 < chars.len()
                && chars[i] == '/' && chars[i+1] == '/' {
                // Find the byte offset for position i
                let byte_pos: usize = line.char_indices()
                    .nth(i).map(|(b, _)| b).unwrap_or(line.len());
                return &line[..byte_pos];
            }
            i += 1;
        }
        line
    }

    /// Count open - close parens for a single line (outside string literals).
    fn paren_depth_of_line(line: &str) -> i32 {
        let mut depth = 0i32;
        let mut in_str = false;
        let mut in_tq = false;
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0usize;
        while i < chars.len() {
            if !in_str && !in_tq && i + 2 < chars.len()
                && chars[i] == '"' && chars[i+1] == '"' && chars[i+2] == '"' {
                in_tq = true;
                i += 3;
                continue;
            }
            if in_tq && i + 2 < chars.len()
                && chars[i] == '"' && chars[i+1] == '"' && chars[i+2] == '"' {
                in_tq = false;
                i += 3;
                continue;
            }
            if !in_tq && chars[i] == '"' { in_str = !in_str; }
            if !in_str && !in_tq {
                match chars[i] {
                    '(' | '[' => depth += 1,
                    ')' | ']' => depth -= 1,
                    _ => {}
                }
            }
            i += 1;
        }
        depth
    }

    // `arrayOf(1.0, 0.0, 0.0, 1.0)` where all elements are float/int literals (no hex) →
    // `listOf(1.0f, 0.0f, 0.0f, 1.0f)` for compatibility with Instance.set / Vertex.set / setClearColor.
    // Does NOT apply if elements look like hex bytes (handled by rewrite_arrayof_byte_literals).
    fn rewrite_arrayof_float_to_listof(line: &str) -> String {
        let needle = "arrayOf(";
        if !line.contains(needle) { return line.to_string(); }

        let chars: Vec<char> = line.chars().collect();
        let mut out = String::with_capacity(line.len() + 32);
        let mut i = 0usize;
        let mut in_tq = false;

        while i < chars.len() {
            // Track triple-quote strings
            if i + 2 < chars.len() && chars[i]=='"' && chars[i+1]=='"' && chars[i+2]=='"' {
                in_tq = !in_tq;
                out.push_str("\"\"\""); i += 3; continue;
            }
            if !in_tq {
                let rem: String = chars[i..].iter().collect();
                if rem.starts_with(needle) {
                    let inner_start = i + needle.len();
                    let mut depth = 0i32;
                    let mut inner_end = None;
                    let mut k = inner_start;
                    while k < chars.len() {
                        match chars[k] {
                            '(' => depth += 1,
                            ')' => { if depth == 0 { inner_end = Some(k); break; } depth -= 1; }
                            _ => {}
                        }
                        k += 1;
                    }
                    if let Some(inner_end) = inner_end {
                        let inner: String = chars[inner_start..inner_end].iter().collect();
                        let elements = split_args(&inner);
                        // Check if all elements are numeric float/int literals (not hex, not strings)
                        let all_numeric = !elements.is_empty() && elements.iter().all(|e| {
                            let e = e.trim().trim_end_matches(',').trim();
                            if e.starts_with("0x") || e.starts_with("0X") { return false; }
                            // Must be a numeric literal (possibly negative, possibly with dot)
                            let e = e.strip_prefix('-').unwrap_or(e);
                            e.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '_')
                                && !e.is_empty()
                        });
                        if all_numeric {
                            let float_list = convert_to_float_list(&inner);
                            out.push_str("listOf(");
                            out.push_str(&float_list);
                            out.push(')');
                            i = inner_end + 1;
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

    // `val name = arrayOf(0x89, 0x50, ...)` where all elements are hex/byte literals →
    // `val name = byteArrayOf(0x89.toByte(), 0x50.toByte(), ...)` for ByteArray compatibility.
    fn rewrite_arrayof_byte_literals(line: &str) -> String {
        // Pattern: `= arrayOf(...)` where ALL elements are hex literals (0x...)
        let needle = "arrayOf(";
        if !line.contains(needle) { return line.to_string(); }
        // Only apply if this looks like a bytes assignment (contains hex literals)
        if !line.contains("0x") && !line.contains("0X") { return line.to_string(); }

        // Find `arrayOf(`
        let Some(start) = line.find(needle) else { return line.to_string(); };
        let inner_start = start + needle.len();
        let chars: Vec<char> = line.chars().collect();

        let mut depth = 0i32;
        let mut inner_end = None;
        let mut j = inner_start;
        while j < chars.len() {
            match chars[j] {
                '(' => depth += 1,
                ')' => {
                    if depth == 0 { inner_end = Some(j); break; }
                    depth -= 1;
                }
                _ => {}
            }
            j += 1;
        }
        let Some(inner_end) = inner_end else { return line.to_string(); };
        let inner: String = chars[inner_start..inner_end].iter().collect();
        let elements = split_args(&inner);

        // Check if all elements are hex literals (possibly with trailing comma)
        let all_hex = elements.iter().all(|e| {
            let e = e.trim().trim_end_matches(',').trim();
            e.starts_with("0x") || e.starts_with("0X")
        });
        if !all_hex { return line.to_string(); }

        // Convert to byteArrayOf(...toByte())
        let byte_elements: Vec<String> = elements.iter()
            .filter(|e| !e.trim().is_empty())
            .map(|e| {
                let e = e.trim().trim_end_matches(',').trim();
                format!("{}.toByte()", e)
            })
            .collect();

        let before = &line[..start];
        let after: String = chars[inner_end + 1..].iter().collect();
        format!("{}byteArrayOf({}){}", before, byte_elements.join(", "), after)
    }

    // `Quad(arrayOf(x1, y1), arrayOf(x2, y2))` → `Quad(listOf(x1f, y1f), listOf(x2f, y2f))`
    // The Kotlin Quad constructor takes (min: List<Float>, max: List<Float>).
    fn rewrite_quad_arrayof_to_listof(line: &str) -> String {
        let needle = "Quad(arrayOf(";
        if !line.contains(needle) { return line.to_string(); }
        // Find `Quad(` start
        let Some(start) = line.find("Quad(") else { return line.to_string(); };
        let call_start = start + "Quad(".len();
        let chars: Vec<char> = line.chars().collect();

        // Find the closing paren for Quad(
        let mut depth = 0i32;
        let mut call_end = None;
        let mut j = call_start;
        while j < chars.len() {
            match chars[j] {
                '(' => depth += 1,
                ')' => {
                    if depth == 0 { call_end = Some(j); break; }
                    depth -= 1;
                }
                _ => {}
            }
            j += 1;
        }
        let Some(call_end) = call_end else { return line.to_string(); };
        let inner: String = chars[call_start..call_end].iter().collect();
        let parts = split_args(&inner);
        if parts.len() != 2 { return line.to_string(); }

        let min_arg = parts[0].trim();
        let max_arg = parts[1].trim();

        fn arrayof_to_listof_floats(s: &str) -> String {
            if s.starts_with("arrayOf(") && s.ends_with(')') {
                let inner = &s["arrayOf(".len()..s.len() - 1];
                let floats = inner.split(',').map(|v| {
                    let v = v.trim();
                    if v.ends_with('f') { v.to_string() }
                    else if v.contains('.') { format!("{}f", v) }
                    else { format!("{}.0f", v) }
                }).collect::<Vec<_>>().join(", ");
                format!("listOf({})", floats)
            } else {
                s.to_string()
            }
        }

        let min_out = arrayof_to_listof_floats(min_arg);
        let max_out = arrayof_to_listof_floats(max_arg);
        let before = &line[..start];
        let after: String = chars[call_end + 1..].iter().collect();
        format!("{}Quad({}, {}){}", before, min_out, max_out, after)
    }

    // `Shader.new([...])` → `Shader.compose(listOf(...))`
    // The Kotlin API exposes `Shader.compose(parts: List<String>)`.
    fn rewrite_shader_new_to_compose(line: &str) -> String {
        let needle = "Shader.new(";
        if !line.contains(needle) { return line.to_string(); }
        // Replace `Shader.new([` → `Shader.compose(listOf(` and the matching `])` → `))`
        // After rewrite_bracket_array_to_arrayof, `[...]` becomes `arrayOf(...)`.
        // So `Shader.new(arrayOf(...)` → `Shader.compose(listOf(...)`
        line.replace("Shader.new(arrayOf(", "Shader.compose(listOf(")
    }

    // `TextureRegion.from([x,y,w,h]).withStride(N).withRows(M)` →
    // `TextureRegionMobile(xu, yu, 0u, wu, hu, 0u, Nu, Mu)`
    // Also handles plain `TextureRegion` → `TextureRegionMobile` renames.
    fn rewrite_texture_region_name(line: &str) -> String {
        let line = line.replace("TextureRegion.from(", "TextureRegionMobile(")
            .replace("TextureRegion(", "TextureRegionMobile(");

        // Handle `TextureRegionMobile(arrayOf(x, y, w, h)).withStride(S).withRows(R)` →
        // `TextureRegionMobile(xu, yu, 0u, wu, hu, 0u, Su, Ru)`
        if line.contains("TextureRegionMobile(arrayOf(") {
            if let Some(result) = rewrite_texture_region_builder(&line) {
                return result;
            }
        }
        line
    }

    fn rewrite_texture_region_builder(line: &str) -> Option<String> {
        let needle = "TextureRegionMobile(arrayOf(";
        let start = line.find(needle)?;
        let array_inner_start = start + needle.len();
        let chars: Vec<char> = line.chars().collect();

        // Find closing paren of arrayOf
        let mut depth = 0i32;
        let mut array_end = None;
        let mut j = array_inner_start;
        while j < chars.len() {
            match chars[j] {
                '(' => depth += 1,
                ')' => {
                    if depth == 0 { array_end = Some(j); break; }
                    depth -= 1;
                }
                _ => {}
            }
            j += 1;
        }
        let array_end = array_end?;
        // After arrayOf(...) should be `)` closing TextureRegionMobile, then optional builder chain
        let after_array: String = chars[array_end + 1..].iter().collect();
        let after_array = after_array.trim_start_matches(')');

        let coords_str: String = chars[array_inner_start..array_end].iter().collect();
        let coords = split_args(&coords_str);
        if coords.len() < 4 { return None; }

        let x = to_uint_arg(coords[0].trim());
        let y = to_uint_arg(coords[1].trim());
        let w = to_uint_arg(coords[2].trim());
        let h = to_uint_arg(coords[3].trim());

        // Parse optional .withStride(N).withRows(M) chain
        let mut bytes_per_row = "null".to_string();
        let mut rows_per_image = "null".to_string();
        let mut remaining = after_array.trim_start();
        while remaining.starts_with(".with") {
            if let Some(r) = remaining.strip_prefix(".withStride(") {
                let close = r.find(')')?;
                bytes_per_row = to_uint_arg(r[..close].trim());
                remaining = r[close + 1..].trim_start();
            } else if let Some(r) = remaining.strip_prefix(".withRows(") {
                let close = r.find(')')?;
                rows_per_image = to_uint_arg(r[..close].trim());
                remaining = r[close + 1..].trim_start();
            } else {
                break;
            }
        }

        let before = &line[..start];
        Some(format!(
            "{}TextureRegionMobile({}, {}, 0u, {}, {}, 0u, {}, {}){}",
            before, x, y, w, h, bytes_per_row, rows_per_image, remaining
        ))
    }

    // `texture.writeRegion(bytes, arrayOf(x, y, w, h))` →
    // `texture.writeRegion(bytes, TextureRegionMobile(x.toUInt(), y.toUInt(), 0u, w.toUInt(), h.toUInt(), 0u, null, null))`
    // Also converts `bytes` to ByteArray if it's Array<Int>.
    fn rewrite_write_region_array_arg(line: &str) -> String {
        let needle = "writeRegion(";
        let Some(start) = line.find(needle) else { return line.to_string(); };
        let call_start = start + needle.len();
        let chars: Vec<char> = line.chars().collect();

        // Find the closing paren for the whole writeRegion call
        let mut depth = 0i32;
        let mut call_end = None;
        let mut j = call_start;
        while j < chars.len() {
            match chars[j] {
                '(' => depth += 1,
                ')' => {
                    if depth == 0 { call_end = Some(j); break; }
                    depth -= 1;
                }
                _ => {}
            }
            j += 1;
        }
        let Some(call_end) = call_end else { return line.to_string(); };
        let inner: String = chars[call_start..call_end].iter().collect();
        let parts = split_args(&inner);
        if parts.len() != 2 { return line.to_string(); }

        let bytes_arg = parts[0].trim();
        let region_arg = parts[1].trim();

        // Only convert if region is an arrayOf(...)
        if !region_arg.starts_with("arrayOf(") || !region_arg.ends_with(')') {
            return line.to_string();
        }

        let array_inner = &region_arg["arrayOf(".len()..region_arg.len() - 1];
        let coords = split_args(array_inner);
        if coords.len() < 4 { return line.to_string(); }

        let x = to_uint_arg(coords[0].trim());
        let y = to_uint_arg(coords[1].trim());
        let w = to_uint_arg(coords[2].trim());
        let h = to_uint_arg(coords[3].trim());

        let before = &line[..start];
        let after: String = chars[call_end + 1..].iter().collect();

        // bytes arg: if it's Array<Int> pattern, note we can't tell at transpile time;
        // just pass through — ByteArray should come from rewrite_array_fill or context
        format!(
            "{}writeRegion({}, TextureRegionMobile({}, {}, 0u, {}, {}, 0u, null, null)){}",
            before, bytes_arg, x, y, w, h, after
        )
    }

    // `renderer.unregisterTexture(id)` → `renderer.unregisterTexture(id.id)` when
    // `id` is a local variable (not a ULong literal). The extension method accepts
    // TextureId directly, so this rewrite is actually handled by the extension.
    // We leave it as-is — the extension method covers it.
    // (No rewrite needed; the extension method makes `unregisterTexture(id: TextureId)` work.)
    fn rewrite_unregister_texture_id(line: &str) -> String {
        line.to_string()
    }

    // `m.setInstanceCount(1_000_000)` → `m.setInstanceCount(1_000_000u)`
    // The Kotlin API takes UInt.
    fn rewrite_set_instance_count_uint(line: &str) -> String {
        let needle = "setInstanceCount(";
        if let Some(start) = line.find(needle) {
            let arg_start = start + needle.len();
            let chars: Vec<char> = line.chars().collect();
            let mut depth = 0i32;
            let mut arg_end = None;
            let mut j = arg_start;
            while j < chars.len() {
                match chars[j] {
                    '(' => depth += 1,
                    ')' => {
                        if depth == 0 { arg_end = Some(j); break; }
                        depth -= 1;
                    }
                    _ => {}
                }
                j += 1;
            }
            if let Some(arg_end) = arg_end {
                let arg: String = chars[arg_start..arg_end].iter().collect();
                let arg = arg.trim();
                // Already a UInt literal?
                if arg.ends_with('u') { return line.to_string(); }
                // Numeric literal (possibly with underscores)
                if arg.chars().all(|c| c.is_ascii_digit() || c == '_') {
                    let before = &line[..start];
                    let after: String = chars[arg_end + 1..].iter().collect();
                    return format!("{}setInstanceCount({}u){}", before, arg, after);
                }
            }
        }
        line.to_string()
    }

    // Convert `Instance().set("key", arrayOf(...))` where the arrayOf value
    // should be a `List<Float>`. Since we added extension methods for
    // `Instance.set(key, List<Float>)` and `Vertex.set(key, List<Float>)`,
    // we just need to convert `arrayOf(0.25, 0.10)` → `listOf(0.25f, 0.10f)`.
    fn rewrite_instance_set_vertex_value(line: &str) -> String {
        // Look for `.set("key", arrayOf(...))`
        let needle = ".set(";
        if !line.contains(needle) || !line.contains("arrayOf(") { return line.to_string(); }

        let chars: Vec<char> = line.chars().collect();
        let mut out = String::with_capacity(line.len());
        let mut i = 0usize;

        while i < chars.len() {
            // Look for `.set(`
            let rem: String = chars[i..].iter().collect();
            if rem.starts_with(".set(") {
                // find the set call's closing paren
                let set_start = i + ".set(".len();
                let mut depth2 = 0i32;
                let mut set_end = None;
                let mut k = set_start;
                while k < chars.len() {
                    match chars[k] {
                        '(' => depth2 += 1,
                        ')' => {
                            if depth2 == 0 { set_end = Some(k); break; }
                            depth2 -= 1;
                        }
                        _ => {}
                    }
                    k += 1;
                }
                if let Some(set_end) = set_end {
                    let inner: String = chars[set_start..set_end].iter().collect();
                    let parts = split_args(&inner);
                    if parts.len() == 2 {
                        let key_arg = parts[0].trim();
                        let val_arg = parts[1].trim();
                        // Convert arrayOf(doubles) → listOf(floats)
                        if val_arg.starts_with("arrayOf(") && val_arg.ends_with(')') {
                            let floats_str = &val_arg["arrayOf(".len()..val_arg.len() - 1];
                            let list_floats = convert_to_float_list(floats_str);
                            out.push_str(&format!(".set({}, listOf({}))", key_arg, list_floats));
                            i = set_end + 1;
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

    // Drop lines that are `val _ = ...` (Kotlin reserves `_`).
    // Called from js_to_kotlin per-line AFTER rewrite_line.
    fn drop_underscore_val(line: &str) -> String {
        let trimmed = line.trim_start();
        if trimmed.starts_with("val _ =") || trimmed == "val _" {
            return String::new();
        }
        line.to_string()
    }

    // `chain.levels()[N]` → `chain.level(Nu)`
    // The uniffi mobile API exposes `level(index: UInt): ByteArray` not `levels(): List<ByteArray>`.
    fn rewrite_levels_indexing(line: &str) -> String {
        // Pattern: `.levels()[N]` where N is a non-negative integer literal
        let needle = ".levels()[";
        let Some(start) = line.find(needle) else { return line.to_string(); };
        let idx_start = start + needle.len();
        let chars: Vec<char> = line.chars().collect();
        let mut idx_end = idx_start;
        while idx_end < chars.len() && (chars[idx_end].is_ascii_digit() || chars[idx_end] == '_') {
            idx_end += 1;
        }
        if idx_end >= chars.len() || chars[idx_end] != ']' { return line.to_string(); }
        let index_str: String = chars[idx_start..idx_end].iter().collect();
        let index_str = index_str.replace('_', "");
        let before = &line[..start];
        let after: String = chars[idx_end + 1..].iter().collect();
        format!("{}.level({}u){}", before, index_str, after)
    }

    // Expand `val (a, b) = expr` destructuring into two separate `val` lines
    // since Kotlin's destructuring only works for `data class` Pair / Triple.
    // Returns `Some(expanded)` if it matched, `None` otherwise.
    fn expand_destructuring_val(line: &str) -> Option<String> {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("val (") { return None; }
        // Pattern: `val (a, b) = expr`
        let rest = &trimmed["val (".len()..];
        let close = rest.find(')')?;
        let names_str = &rest[..close];
        let after_close = rest[close + 1..].trim_start();
        if !after_close.starts_with('=') { return None; }
        let rhs = after_close[1..].trim();
        let names: Vec<&str> = names_str.split(',').map(|s| s.trim()).collect();
        if names.len() < 2 { return None; }
        let indent_len = line.len() - trimmed.len();
        let indent = &line[..indent_len];
        // Emit: `val tmp_size = expr; val a = tmp_size.field0; val b = tmp_size.field1`
        // For baseSize() which returns Size, use .width and .height
        let tmp = "tmp_size";
        let mut out = format!("{}val {} = {}", indent, tmp, rhs);
        // Try to use named fields based on context
        let field_names = if rhs.contains("baseSize") || rhs.contains("size") {
            vec!["width", "height"]
        } else {
            // Generic: use component1(), component2()
            (0..names.len()).map(|i| match i {
                0 => "component1()",
                1 => "component2()",
                2 => "component3()",
                _ => "componentN()",
            }).collect::<Vec<_>>()
        };
        for (idx, name) in names.iter().enumerate() {
            let field = field_names.get(idx).copied().unwrap_or("componentN()");
            let accessor = if field.ends_with("()") {
                format!("{}.{}", tmp, field)
            } else {
                format!("{}.{}", tmp, field)
            };
            out.push_str(&format!("\n{}val {} = {}", indent, name, accessor));
        }
        Some(out)
    }
}
