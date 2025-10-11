mod convert {
    use crate::{js_objectize_sampleroptions_literal, js_region_new_to_array, simplify_js_size_from};

    #[derive(Copy, Clone, Debug)]
    enum Lang {
        Js,
        Py,
    }

    pub fn to_js(items: &[(String, bool)]) -> String {
        convert(items, Lang::Js)
    }

    pub fn to_py(items: &[(String, bool)]) -> String {
        convert(items, Lang::Py)
    }

    fn is_ident_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }

    // Helper: convert common Rust Vec constructs to JS/Python arrays
    fn convert_vec_syntax(s: &str) -> String {
        let mut out = s.replace("Vec::new()", "[]");
        out = out.replace("vec![", "[");
        out = out.replace("vec ![", "["); // tolerate space
        out
    }

    fn snake_to_camel(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        let mut upper = false;
        for ch in s.chars() {
            if ch == '_' {
                upper = true;
            } else if upper {
                out.push(ch.to_ascii_uppercase());
                upper = false;
            } else {
                out.push(ch);
            }
        }
        out
    }

    fn camelize_method_calls_js(line: &str) -> String {
        let mut out = String::with_capacity(line.len());
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0usize;
        while i < chars.len() {
            let c = chars[i];
            out.push(c);
            if c == '.' {
                // capture identifier after '.'
                let start = i + 1;
                let mut j = start;
                while j < chars.len() && is_ident_char(chars[j]) {
                    j += 1;
                }
                if j > start {
                    let ident: String = chars[start..j].iter().collect();
                    let camel = snake_to_camel(&ident);
                    out.push_str(&camel);
                    i = j; // skip consumed ident; next loop pushes current char again, so avoid double push
                    continue;
                }
            }
            i += 1;
        }
        out
    }

    fn replace_static_call_to_dot(line: &str) -> String {
        // Replace patterns like Type::method( -> Type.method(
        let mut out = String::new();
        let mut i = 0usize;
        let bytes = line.as_bytes();
        while i < bytes.len() {
            if i + 2 < bytes.len() && bytes[i] == b':' && bytes[i + 1] == b':' {
                // Replace '::' with '.'
                out.push('.');
                i += 2;
                continue;
            }
            out.push(bytes[i] as char);
            i += 1;
        }
        out
    }

    fn strip_refs(s: &str) -> String {
        s.replace('&', "")
    }

    fn strip_trailing_semicolon(s: &str) -> String {
        let t = s.trim_end();
        if let Some(stripped) = t.strip_suffix(';') {
            stripped.to_string()
        } else {
            s.to_string()
        }
    }

    fn ensure_js_semicolon(s: &str) -> String {
        let t = s.trim_end();
        if t.is_empty() {
            return String::new();
        }
        // If it already ends with a semicolon, keep as-is
        if t.ends_with(';') {
            return s.to_string();
        }
        // Do not append semicolons to lines that are clearly mid-expression
        // Common multi-line JS patterns we should respect:
        // - Lines ending with an opening bracket/brace/paren: [, {, (
        // - Lines ending with a comma (array/object items, arg lists): ,
        // - Pure comment lines starting with //
        // - Arrow function headers ending with =>
        let t_no_leading = t.trim_start();
        if t_no_leading.starts_with("//") {
            return s.to_string();
        }
        if t.ends_with("=>") {
            return s.to_string();
        }
        if let Some(last) = t.chars().rev().find(|c| !c.is_whitespace())
            && matches!(last, '[' | '{' | '(' | ',')
        {
            return s.to_string();
        }
        // Otherwise, terminate the statement with a semicolon
        format!("{};", s)
    }

    fn transform_await(line: &str, lang: Lang) -> String {
        // Handle `.await?` -> `await` (JS) or remove (Py). Also drop Rust error `?` in JS/Py.
        let mut out = line.to_string();
        if out.contains(".await?") {
            match lang {
                Lang::Js => {
                    if let Some(pos) = out.find(".await?") {
                        // Preserve left side if present
                        if let Some(eq) = out[..pos].rfind('=') {
                            let (lhs, expr) = out.split_at(eq + 1);
                            let expr = expr.trim();
                            let before_await = &expr[..expr.rfind(".await?").unwrap_or(expr.len())];
                            let mut s = String::new();
                            s.push_str(lhs);
                            s.push(' ');
                            s.push_str("await ");
                            s.push_str(before_await.trim());
                            let tail = &out[pos + ".await?".len()..];
                            s.push_str(tail);
                            out = s;
                        } else {
                            let before_await = &out[..pos];
                            let mut s = String::new();
                            s.push_str("await ");
                            s.push_str(before_await.trim());
                            let tail = &out[pos + ".await?".len()..];
                            s.push_str(tail);
                            out = s;
                        }
                    }
                }
                Lang::Py => {
                    out = out.replace(".await?", "");
                }
            }
        }
        // Remove stray Rust error-propagation '?' for both langs (JS/Py)
        match lang {
            Lang::Js => {
                // Replace common patterns
                let mut s = out.replace(")?;", ");");
                s = s.replace(")?\n", ")\n");
                s = s.replace(")? ", ") ");
                if s.trim_end().ends_with('?') {
                    s = s.trim_end_matches('?').to_string();
                }
                s
            }
            Lang::Py => out.replace('?', ""),
        }
    }

    fn map_assert(line: &str, lang: Lang) -> Option<String> {
        // assert_eq!(a, b);
        let t = line.trim_start();
        if !t.starts_with("assert_eq!(") {
            return None;
        }
        let mut inner = &t["assert_eq!(".len()..];
        if let Some(end) = inner.find(')') {
            inner = &inner[..end];
        }
        let mut parts = inner.splitn(2, ',');
        let a = parts.next().unwrap_or("").trim();
        let b = parts.next().unwrap_or("").trim();
        match lang {
            Lang::Js => Some(format!(
                "if (JSON.stringify({}) !== JSON.stringify({})) {{ throw new Error(\"assert_eq failed\"); }}",
                a, b
            )),
            Lang::Py => Some(format!("assert {} == {}", a, b)),
        }
    }

    fn parse_use_fragmentcolor(line: &str) -> Option<Vec<String>> {
        // Returns a flat list of imported names from `use fragmentcolor::...` syntax
        // Supports nested paths before a group: e.g., `use fragmentcolor::mesh::{Mesh, Vertex};`
        // and single items: `use fragmentcolor::mesh::Vertex;` or `use fragmentcolor::{Renderer, Target};`
        let t = line.trim();
        if !t.starts_with("use fragmentcolor::") {
            return None;
        }
        // Slice after the prefix and strip a trailing ';' if present
        let mut after = &t["use fragmentcolor::".len()..];
        if let Some(sc) = after.rfind(';') {
            after = &after[..sc];
        }
        let after = after.trim();

        // If there's a brace group anywhere after the prefix, extract names inside the outermost braces
        if let (Some(lc), Some(rc)) = (after.find('{'), after.rfind('}'))
            && lc < rc
        {
            let inside = &after[lc + 1..rc];
            let list: Vec<String> = inside
                .split(',')
                .map(|p| p.trim())
                .filter(|s| !s.is_empty())
                .map(|p| p.rsplit("::").next().unwrap_or("").to_string())
                .filter(|s| !s.is_empty() && s != "self")
                .collect();
            return Some(list);
        }

        // No brace group: take the last path segment as the identifier
        let short = after
            .rsplit("::")
            .next()
            .unwrap_or(after)
            .trim()
            .to_string();
        if short.is_empty() {
            None
        } else {
            Some(vec![short])
        }
    }

    fn handle_let_assignment(
        line: &str,
        lang: Lang,
        py_renames: &mut std::collections::HashMap<String, String>,
        js_renames: &mut std::collections::HashMap<String, String>,
        need_rendercanvas_import: &mut bool,
    ) -> Option<String> {
        let t = line.trim_start();
        if !t.starts_with("let ") {
            return None;
        }
        // strip leading `let` and optional `mut`
        let rest = t.trim_start_matches("let ").trim_start();
        let rest = if let Some(stripped) = rest.strip_prefix("mut ") {
            stripped
        } else {
            rest
        };
        let eq = rest.find('=')?;
        let (lhs, rhs0) = rest.split_at(eq);
        let mut var = lhs.trim();
        // Strip type annotation in `var: Type`
        if let Some(colon) = var.find(':') {
            var = var[..colon].trim();
        }
        let mut rhs = rhs0.trim_start_matches('=').trim().to_string();
        // Remove trailing ';'
        rhs = strip_trailing_semicolon(&rhs);

        // Replace fragmentcolor headless helpers with native canvases
        // Python: fragmentcolor::headless_window([w,h]) -> RenderCanvas(size=(w,h))
        // JS: fragmentcolor.headlessWindow([w,h]) -> (()=>{const c=document.createElement('canvas');c.width=w;c.height=h;return c;})()
        let rhs_lc = rhs.replace("::", ".");
        if rhs_lc.contains("headless_window(") || rhs_lc.contains("headlessWindow(") {
            match lang {
                Lang::Py => {
                    if let (Some(lp), Some(rp)) = (rhs.find('('), rhs.rfind(')')) {
                        let inside = rhs[lp + 1..rp].trim();
                        let inner = inside
                            .trim_start_matches('[')
                            .trim_end_matches(']')
                            .trim_start_matches('(')
                            .trim_end_matches(')')
                            .trim();
                        rhs = format!("RenderCanvas(size=({}))", inner);
                        *need_rendercanvas_import = true;
                        if var == "window" {
                            // rename variable to canvas in Python output and future references
                            py_renames.insert("window".into(), "canvas".into());
                        }
                    }
                }
                Lang::Js => {
                    if let (Some(_), Some(_)) = (rhs.find('('), rhs.rfind(')')) {
                        rhs = "document.createElement('canvas');".to_string();
                        if var == "window" {
                            js_renames.insert("window".into(), "canvas".into());
                        }
                    }
                }
            }
        } else {
            // Type::new(args)
            if let Some(pos) = rhs.find("::new(") {
                let ty = rhs[..pos].trim().rsplit("::").next().unwrap_or("");
                let args_with = &rhs[pos + "::new(".len()..];
                if let Some(endp) = args_with.rfind(')') {
                    let args = &args_with[..endp];
                    rhs = match lang {
                        Lang::Js => format!("new {}({})", ty, args.trim()),
                        Lang::Py => format!("{}({})", ty, args.trim()),
                    };
                }
            }
        }
        // Pass::from_shader(name, shader) -> one-line construct+add
        if rhs.starts_with("Pass::from_shader(")
            || rhs.starts_with("fragmentcolor::Pass::from_shader(")
        {
            // Extract args inside (...)
            if let Some(lp) = rhs.find('(')
                && let Some(rp) = rhs.rfind(')')
            {
                let inside = &rhs[lp + 1..rp];
                let mut parts = inside.splitn(2, ',');
                let a1 = parts.next().unwrap_or("").trim();
                let mut a2 = parts.next().unwrap_or("").trim().to_string();
                a2 = strip_refs(&a2);
                match lang {
                    Lang::Js => {
                        rhs = format!("new Pass({}); {}.addShader({})", a1, var, a2);
                    }
                    Lang::Py => {
                        rhs = format!("Pass({}); {}.add_shader({})", a1, var, a2);
                    }
                }
            }
        }

        // UFCS associated calls remaining: Type::method( -> Type.method(
        rhs = replace_static_call_to_dot(&rhs);
        // Python-specific cleanups: simplify Type.new(args) -> Type(args); Size.from(x) -> x; strip unwrap()
        if let Lang::Py = lang {
            rhs = simplify_py_static_new(&rhs);
            rhs = simplify_py_size_from(&rhs);
            // Convert SamplerOptions { ... } -> { ... } Python dict
            if rhs.contains("SamplerOptions {") {
                rhs = pythonize_sampleroptions_literal(&rhs);
            }
            if rhs.contains(".unwrap()") {
                rhs = rhs.replace(".unwrap()", "");
            }
        }
        // Strip refs '&'
        rhs = strip_refs(&rhs);
        // Await transform and remove '?' for lang
        rhs = transform_await(&rhs, lang);

        // JS: camelize method names after '.'
        if let Lang::Js = lang {
            rhs = camelize_method_calls_js(&rhs);
            // JS fixups:
            // 1) None -> null
            rhs = rhs.replace("None", "null");
            // 2) SamplerOptions { ... } -> { ... }
            if rhs.contains("SamplerOptions {") {
                rhs = js_objectize_sampleroptions_literal(&rhs);
            }
            // 3) Size.from(x) -> x
            rhs = simplify_js_size_from(&rhs);
            // 4) Region.new(a,b,c,d) -> [a,b,c,d]
            rhs = js_region_new_to_array(&rhs);
            // 5) std.fs.read(path) -> "/healthcheck/public/favicon.png" (served by healthcheck server)
            if let Some(i) = rhs.find("std.fs.read(") {
                let before = &rhs[..i];
                let after = &rhs[i + "std.fs.read(".len()..];
                if let Some(rp) = after.find(')') {
                    let tail = &after[rp + 1..];
                    rhs = format!("{}\"/healthcheck/public/favicon.png\"{}", before, tail);
                }
            }
        }
        // Python fix: map std.fs.read(path) -> open(path, "rb").read() in RHS
        if let Lang::Py = lang
            && let Some(i) = rhs.find("std.fs.read(")
        {
            let before = &rhs[..i];
            let after = &rhs[i + "std.fs.read(".len()..];
            if let Some(rp) = after.find(')') {
                let args = &after[..rp];
                let tail = &after[rp + 1..];
                rhs = format!("{}open({}, \"rb\").read(){}", before, args.trim(), tail);
            }
        }
        // Python: convert `.size()` calls on RHS into `.size` property
        if let Lang::Py = lang {
            rhs = replace_py_size_calls(&rhs);
        }

        // Var rename for Python reserved keyword "pass" and window->canvas mapping
        let var_out: &str = match lang {
            Lang::Py => {
                if var == "pass" {
                    py_renames.insert("pass".into(), "rpass".into());
                    "rpass"
                } else if var == "window" && py_renames.get("window").is_some() {
                    "canvas"
                } else {
                    var
                }
            }
            Lang::Js => {
                if var == "window" && js_renames.get("window").is_some() {
                    "canvas"
                } else {
                    var
                }
            }
        };

        // If RHS references the original var (e.g., Pass::from_shader expansion), adjust to var_out for Python
        if let Lang::Py = lang {
            let needle = format!("{}.", var);
            let replacement = format!("{}.", var_out);
            rhs = rhs.replace(&needle, &replacement);
        }

        // Convert Rust vec! macros and Vec::new() in RHS to JS/Python arrays
        rhs = convert_vec_syntax(&rhs);

        // Apply pending variable renames inside RHS for both languages (e.g., window->canvas)
        rhs = match lang {
            Lang::Py => apply_renames_py(&rhs, py_renames),
            Lang::Js => apply_renames_py(&rhs, js_renames),
        };

        let mut line_out = match lang {
            Lang::Js => ensure_js_semicolon(&format!("const {} = {}", var_out, rhs)),
            Lang::Py => format!("{} = {}", var_out, rhs),
        };
        // Python: Convert JS-style '//' comments to Python '#'
        if let Lang::Py = lang
            && let Some(idx) = line_out.find("//")
        {
            let (mut head, tail) = line_out.split_at(idx);
            head = head.trim_end_matches(';').trim_end();
            if head.is_empty() {
                line_out = format!("#{}", &tail[2..]);
            } else {
                line_out = format!("{} #{}", head, &tail[2..]);
            }
        }
        Some(line_out)
    }

    fn apply_renames_py(s: &str, renames: &std::collections::HashMap<String, String>) -> String {
        if renames.is_empty() {
            return s.to_string();
        }
        fn replace_word(src: &str, from: &str, to: &str) -> String {
            let bytes: Vec<char> = src.chars().collect();
            let from_chars: Vec<char> = from.chars().collect();
            let mut out = String::with_capacity(src.len());
            let mut i = 0usize;
            while i < bytes.len() {
                // Try to match `from` at position i with word boundaries
                let end = i + from_chars.len();
                if end <= bytes.len() && bytes[i..end] == from_chars[..] {
                    let left_ok = i == 0 || !super::convert::is_ident_char(bytes[i - 1]);
                    let right_ok = end == bytes.len() || !super::convert::is_ident_char(bytes[end]);
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
        let mut out = s.to_string();
        for (from, to) in renames {
            out = replace_word(&out, from, to);
        }
        out
    }

    // Detect and convert a multi-line Rust raw string passed to Type::new(r#"..."#)
    // into:
    // - JS: const var = new Type(`...`);
    // - Py: var = Type("""...""")
    // Returns (mapped_text, next_index) when a block is consumed, or None if not matched.
    fn try_handle_raw_string_new(
        src: &[String],
        start_idx: usize,
        lang: Lang,
        py_renames: &mut std::collections::HashMap<String, String>,
        _js_renames: &mut std::collections::HashMap<String, String>,
    ) -> Option<(String, usize)> {
        let line = src.get(start_idx)?.as_str();
        let t = line.trim_start();
        if !t.starts_with("let ") {
            return None;
        }
        // Parse 'let [mut] var[: Type] = RHS'
        let mut rest = t.trim_start_matches("let ").trim_start();
        if let Some(r) = rest.strip_prefix("mut ") {
            rest = r.trim_start();
        }
        let eq = rest.find('=')?;
        let (lhs, rhs0) = rest.split_at(eq);
        // var name (strip any ': Type')
        let mut var = lhs.trim();
        if let Some(colon) = var.find(':') {
            var = var[..colon].trim();
        }
        let mut var_out = var.to_string();
        // Python reserved keyword rename
        if let Lang::Py = lang
            && var == "pass"
        {
            py_renames.insert("pass".into(), "rpass".into());
            var_out = "rpass".to_string();
        }
        // RHS on this first line
        let rhs_line = rhs0.trim_start_matches('=').trim();
        // Must look like '<Path>::new('
        let pos_new = rhs_line.find("::new(")?;
        let type_path = rhs_line[..pos_new].trim();
        let ty = type_path
            .rsplit("::")
            .next()
            .unwrap_or(type_path)
            .to_string();
        let after_new = &rhs_line[pos_new + "::new(".len()..];

        // Helper to parse raw opener: r####"  -> returns (hashes, pos_after_quote_in_this_slice)
        fn parse_raw_opener(
            s: &str,
        ) -> Option<(
            usize, /*n_hash*/
            usize, /*pos after opening quote in s*/
        )> {
            let s_trim = s.trim_start();
            let off = s.len() - s_trim.len();
            let mut chars = s_trim.chars();
            let first = chars.next()?;
            if first != 'r' {
                return None;
            }
            // Count '#'
            let mut n_hash = 0usize;
            let mut idx = 1usize; // after 'r'
            let s_bytes: Vec<char> = s_trim.chars().collect();
            while idx < s_bytes.len() && s_bytes[idx] == '#' {
                n_hash += 1;
                idx += 1;
            }
            if idx >= s_bytes.len() || s_bytes[idx] != '"' {
                return None;
            }
            // Position after opening quote within original s
            let pos_after_quote = off + idx + 1;
            Some((n_hash, pos_after_quote))
        }

        // Try to find opener on this same line after 'new('
        let opener = parse_raw_opener(after_new)?;
        let (n_hash, pos_after_quote_in_after_new) = opener;

        // Collect body lines until closing '"###...#'
        let closing = {
            let mut s = String::from("\"");
            for _ in 0..n_hash {
                s.push('#');
            }
            s
        };

        let mut body_lines: Vec<String> = Vec::new();
        let first_tail = &after_new[pos_after_quote_in_after_new..];
        if !first_tail.is_empty() {
            // Content on same line after the opening quote
            // Stop early if the closing is also on this line
            if let Some(pos) = first_tail.find(&closing) {
                // All inside one line raw string
                let content = &first_tail[..pos];
                body_lines.push(content.to_string());
                // next_idx is still current line (consumed only 1 line)
                let mapped = match lang {
                    Lang::Js => format!(
                        "const {} = new {}(`\n{}\n`);",
                        var_out,
                        ty,
                        body_lines.join("\n")
                    ),
                    Lang::Py => format!(
                        "{} = {}(\"\"\"\n{}\n\"\"\")",
                        var_out,
                        ty,
                        body_lines.join("\n")
                    ),
                };
                return Some((mapped, start_idx + 1));
            } else {
                body_lines.push(first_tail.to_string());
            }
        }

        // Scan subsequent lines until closing marker is found
        let mut j = start_idx + 1;
        while j < src.len() {
            let l = &src[j];
            if let Some(pos) = l.find(&closing) {
                let content = &l[..pos];
                body_lines.push(content.to_string());
                // Done; compute output and return next index after closing line
                let mapped = match lang {
                    Lang::Js => format!(
                        "const {} = new {}(`\n{}\n`);",
                        var_out,
                        ty,
                        body_lines.join("\n")
                    ),
                    Lang::Py => format!(
                        "{} = {}(\"\"\"\n{}\n\"\"\")",
                        var_out,
                        ty,
                        body_lines.join("\n")
                    ),
                };
                return Some((mapped, j + 1));
            } else {
                body_lines.push(l.clone());
                j += 1;
            }
        }
        // If we get here, we didn't find a proper terminator; don't transform.
        None
    }

    fn convert(items: &[(String, bool)], lang: Lang) -> String {
        use std::collections::HashMap;
        // Collect visible lines only
        let mut src: Vec<String> = Vec::new();
        for (t, hidden) in items {
            if !*hidden {
                src.push(t.clone());
            }
        }
        // Enforce no visible into()
        for line in &src {
            if line.contains(".into(") || line.contains(".into()") || line.contains(" into(") {
                panic!(
                    "Visible .into() found in an example. Hide it with '#' or remove it: \n {}",
                    line
                );
            }
        }
        // Enforce no visible pollster
        for line in &src {
            if line.contains("pollster") {
                panic!(
                    "Visible pollster found in an example. Hide it with '#' or remove it: \n {}",
                    line
                );
            }
        }
        // Enforce no visible pollster
        for line in &src {
            if line.contains("assert_eq") {
                panic!(
                    "Visible assert_eq found in an example. Hide it with '#' or remove it: \n {}",
                    line
                );
            }
        }

        let mut out: Vec<String> = Vec::new();
        let mut py_renames: HashMap<String, String> = HashMap::new();
        let mut js_renames: HashMap<String, String> = HashMap::new();
        let mut need_rendercanvas_import: bool = false;

        let mut idx = 0usize;
        while idx < src.len() {
            let s = &src[idx];
            let t = s.trim();
            if t.is_empty() {
                out.push(String::new());
                idx += 1;
                continue;
            }

            // Drop Rust-only windowing note for non-Rust outputs
            // "We officially support Winit. Check the examples folder for details." doesn't apply to JS/Python.
            if t.contains("We officially support Winit") {
                idx += 1;
                continue;
            }

            // Special-case: let var = Type::new(r#"..."#) with multi-line raw string
            if let Some((mapped, next_idx)) =
                try_handle_raw_string_new(&src, idx, lang, &mut py_renames, &mut js_renames)
            {
                out.push(mapped);
                idx = next_idx;
                continue;
            }

            // Imports from fragmentcolor
            if let Some(list) = parse_use_fragmentcolor(t) {
                // remove Target, WindowTarget, TextureTarget, Size, and SamplerOptions from imports
                let list: Vec<String> = list
                    .into_iter()
                    .filter(|name| {
                        name != "Target"
                            && name != "WindowTarget"
                            && name != "TextureTarget"
                            && name != "Size"
                            && name != "SamplerOptions"
                            && name != "VertexValue"
                    })
                    .collect();
                match lang {
                    Lang::Js => out.push(format!(
                        "import {{ {} }} from \"fragmentcolor\";",
                        list.join(", ")
                    )),
                    Lang::Py => {
                        if !list.is_empty() {
                            out.push(format!("from fragmentcolor import {}", list.join(", ")))
                        }
                    }
                }
                idx += 1;
                continue;
            }

            // Sanitize any raw Python import lines
            if let Lang::Py = lang {
                if t.starts_with("from fragmentcolor import ") {
                    let raw = t.trim_start_matches("from fragmentcolor import ").trim();
                    let mut names: Vec<String> =
                        raw.split(',').map(|s| s.trim().to_string()).collect();
                    names.retain(|name| {
                        name != "Target"
                            && name != "WindowTarget"
                            && name != "TextureTarget"
                            && name != "Size"
                            && name != "SamplerOptions"
                    });
                    if !names.is_empty() {
                        out.push(format!("from fragmentcolor import {}", names.join(", ")));
                    }
                    idx += 1;
                    continue;
                }
                if t.starts_with("from fragmentcolor import {")
                    && let (Some(lb), Some(rb)) = (t.find('{'), t.find('}'))
                {
                    let inside = &t[lb + 1..rb];
                    let mut names: Vec<String> =
                        inside.split(',').map(|s| s.trim().to_string()).collect();
                    names.retain(|name| {
                        name != "Target"
                            && name != "WindowTarget"
                            && name != "TextureTarget"
                            && name != "Size"
                            && name != "SamplerOptions"
                    });
                    if !names.is_empty() {
                        out.push(format!("from fragmentcolor import {}", names.join(", ")));
                    }
                    idx += 1;
                    continue;
                }
            }

            // Map assert_eq!
            if let Some(mapped) = map_assert(t, lang) {
                out.push(match lang {
                    Lang::Js => ensure_js_semicolon(&mapped),
                    Lang::Py => mapped,
                });
                idx += 1;
                continue;
            }

            // let assignments (single-line cases)
            if let Some(mapped) = handle_let_assignment(
                t,
                lang,
                &mut py_renames,
                &mut js_renames,
                &mut need_rendercanvas_import,
            ) {
                out.push(mapped);
                idx += 1;
                continue;
            }

            // General expression/method calls
            let mut line = s.to_string();

            // 1) UFCS static call -> dot first
            line = replace_static_call_to_dot(&line);
            // JS-specific early fixups on raw line
            if let Lang::Js = lang {
                // None -> null
                line = line.replace("None", "null");
                // SamplerOptions { ... } -> { ... }
                if line.contains("SamplerOptions {") {
                    line = js_objectize_sampleroptions_literal(&line);
                }
                // Size.from(x) -> x
                line = simplify_js_size_from(&line);
                // Region.new(a,b,c,d) -> [a,b,c,d]
                line = js_region_new_to_array(&line);
                // std.fs.read("...") -> "/healthcheck/public/favicon.png"
                if let Some(i) = line.find("std.fs.read(") {
                    let before = &line[..i];
                    let after = &line[i + "std.fs.read(".len()..];
                    if let Some(rp) = after.find(')') {
                        let tail = &after[rp + 1..];
                        line = format!("{}\"/healthcheck/public/favicon.png\"{}", before, tail);
                    }
                }
            }

            // 1.1) Python: simplify Type.new(args) -> Type(args)
            if let Lang::Py = lang {
                line = simplify_py_static_new(&line);
                // Map Size.from([w,h]) or Size.from((w,h)) -> (w,h)
                line = simplify_py_size_from(&line);
                // Convert SamplerOptions literal to dict
                if line.contains("SamplerOptions {") {
                    line = pythonize_sampleroptions_literal(&line);
                }
                // Drop Result-style unwrap() noise
                if line.contains(".unwrap()") {
                    line = line.replace(".unwrap()", "");
                }
            }

            // 1.1) Convert Vec syntax (vec![], Vec::new()) into JS/Py arrays
            line = convert_vec_syntax(&line);

            // 2) Drop explicit module prefix for JS/Py when present: fragmentcolor.Shader -> Shader
            if matches!(lang, Lang::Js | Lang::Py) {
                line = line.replace("fragmentcolor.Shader", "Shader");
            }

            // 3) Handling for Shader::default();
            if line.contains("Shader::default()")
                || line.contains("Shader.default()")
                || line.contains("fragmentcolor::Shader::default()")
                || line.contains("fragmentcolor::Shader.default()")
            {
                line = match lang {
                    Lang::Js => line
                        .replace("Shader::default()", "new Shader(\"\")")
                        .replace("fragmentcolor::Shader::default()", "new Shader(\"\")")
                        .replace("Shader.default()", "new Shader(\"\")")
                        .replace("fragmentcolor::Shader.default()", "new Shader(\"\")"),
                    Lang::Py => {
                        // Add import at the top later if needed
                        need_rendercanvas_import = true;
                        line.replace("Shader::default()", "Shader(\"\")")
                            .replace("fragmentcolor::Shader::default()", "Shader(\"\")")
                            .replace("Shader.default()", "Shader(\"\")")
                            .replace("fragmentcolor::Shader.default()", "Shader(\"\")")
                    }
                };
            }

            // 4) Strip refs
            // Python fix: map std.fs.read(path) -> open(path, "rb").read()
            if let Lang::Py = lang
                && let Some(i) = line.find("std.fs.read(")
            {
                let before = &line[..i];
                let after = &line[i + "std.fs.read(".len()..];
                if let Some(rp) = after.find(')') {
                    let args = &after[..rp];
                    let tail = &after[rp + 1..];
                    line = format!("{}open({}, \"rb\").read(){}", before, args.trim(), tail);
                }
            }
            line = strip_refs(&line);

            // 5) Await / remove error '?' artifacts
            line = transform_await(&line, lang);

            // 6) Python size property + comment conversion
            if let Lang::Py = lang {
                // Robustly convert `.size()` (with optional whitespace) into `.size`
                line = replace_py_size_calls(&line);
                // Convert JS-style '//' comments to Python '#'
                if let Some(idx) = line.find("//") {
                    let (mut head, tail) = line.split_at(idx);
                    // Strip any trailing semicolon immediately before comment
                    head = head.trim_end_matches(';').trim_end();
                    if head.is_empty() {
                        // Pure comment line: no leading space before '#'
                        line = format!("#{}", &tail[2..]);
                    } else {
                        line = format!("{} #{}", head, &tail[2..]);
                    }
                }
            }

            // 7) JS camelize methods
            if let Lang::Js = lang {
                line = camelize_method_calls_js(&line);
            }

            // 8) Language-specific trailing cleanup
            match lang {
                Lang::Js => {
                    line = ensure_js_semicolon(&line);
                }
                Lang::Py => {
                    line = strip_trailing_semicolon(&line).replace('?', "");
                }
            }

            // 9) Apply var renames in Python/JS lines after we possibly introduced references
            match lang {
                Lang::Py => {
                    line = apply_renames_py(&line, &py_renames);
                }
                Lang::Js => {
                    line = apply_renames_py(&line, &js_renames);
                }
            }

            out.push(line);
            idx += 1;
        }

        // Prepend RenderCanvas import if needed for Python examples
        if need_rendercanvas_import {
            let mut out2 = Vec::with_capacity(out.len() + 1);
            out2.push("from rendercanvas.auto import RenderCanvas, loop".to_string());
            out2.extend(out);
            out = out2;
        }

        // Normalize ending newline joining in caller
        out.join("\n")
    }

    // Replace occurrences of Type.new(args) -> Type(args) in a best-effort way (no regex).
    fn simplify_py_static_new(line: &str) -> String {
        let mut out = String::with_capacity(line.len());
        let bytes: Vec<char> = line.chars().collect();
        let mut i = 0usize;
        while i < bytes.len() {
            // Look for ".new(" pattern
            if i + 5 <= bytes.len() && bytes[i..i + 5] == ['.', 'n', 'e', 'w', '('] {
                // Backtrack to the start of the identifier
                let mut j = i;
                while j > 0 && super::convert::is_ident_char(bytes[j - 1]) {
                    j -= 1;
                }
                // Rewrite: keep the identifier, drop ".new"
                // out already has content up to j; ensure we don't duplicate
                let prefix_len = out.chars().count();
                if prefix_len < j {
                    out.push_str(&bytes[prefix_len..j].iter().collect::<String>());
                }
                // Skip ".new"
                i += 4; // will be incremented by loop to land on '('
                continue;
            }
            out.push(bytes[i]);
            i += 1;
        }
        out
    }

    // Convert Size.from(arg) -> arg (strip the wrapper) for Python.
    fn simplify_py_size_from(line: &str) -> String {
        let needle = "Size.from(";
        if let Some(start) = line.find(needle) {
            // Find matching closing parenthesis
            let mut depth = 0i32;
            let mut end = start + needle.len();
            let chars: Vec<char> = line.chars().collect();
            let mut i = end;
            while i < chars.len() {
                let c = chars[i];
                if c == '(' {
                    depth += 1;
                }
                if c == ')' {
                    if depth == 0 {
                        end = i;
                        break;
                    }
                    depth -= 1;
                }
                i += 1;
            }
            if end > start + needle.len() {
                let before = &line[..start];
                let inner = &line[start + needle.len()..end];
                let after = &line[end + 1..]; // skip ')'
                return format!("{}{}{}", before, inner.trim(), after);
            }
        }
        line.to_string()
    }

    // Replace `.size()` (allowing whitespace as `.size ( )`) with `.size` in Python output
    fn replace_py_size_calls(line: &str) -> String {
        let chars: Vec<char> = line.chars().collect();
        let mut out = String::with_capacity(line.len());
        let mut i = 0usize;
        while i < chars.len() {
            if chars[i] == '.' {
                // optionally skip whitespace after '.'
                let mut j = i + 1;
                while j < chars.len() && chars[j].is_whitespace() {
                    j += 1;
                }
                // match 'size'
                let name = ['s', 'i', 'z', 'e'];
                if j + name.len() <= chars.len() && chars[j..j + name.len()] == name {
                    let mut k = j + name.len();
                    // skip whitespace before '('
                    while k < chars.len() && chars[k].is_whitespace() {
                        k += 1;
                    }
                    if k < chars.len() && chars[k] == '(' {
                        k += 1;
                        // skip whitespace inside parens
                        while k < chars.len() && chars[k].is_whitespace() {
                            k += 1;
                        }
                        if k < chars.len() && chars[k] == ')' {
                            // success: write ".size" and skip to after ')'
                            out.push('.');
                            out.push_str("size");
                            i = k + 1;
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

    // Pythonize a Rust literal: SamplerOptions { repeat_x: true, ... }
    fn pythonize_sampleroptions_literal(line: &str) -> String {
        let needle = "SamplerOptions {";
        if let Some(start) = line.find(needle) {
            let chars: Vec<char> = line.chars().collect();
            let mut depth = 0i32;
            let mut i = start + needle.len();
            let mut end = i;
            while i < chars.len() {
                let c = chars[i];
                if c == '{' {
                    depth += 1;
                }
                if c == '}' {
                    if depth == 0 {
                        end = i;
                        break;
                    }
                    depth -= 1;
                }
                i += 1;
            }
            if end > start {
                let before = &line[..start];
                let inside = &line[start + needle.len()..end];
                let after = &line[end + 1..];
                let mut fields: Vec<String> = Vec::new();
                for part in inside.split(',') {
                    let p = part.trim();
                    if p.is_empty() {
                        continue;
                    }
                    if let Some(colon) = p.find(':') {
                        let key = p[..colon].trim();
                        let mut val = p[colon + 1..].trim().to_string();
                        val = val.replace("true", "True").replace("false", "False");
                        if val.starts_with("CompareFunction::") {
                            let name = val.trim_start_matches("CompareFunction::");
                            val = format!("\"{}\"", name);
                        }
                        fields.push(format!("\"{}\": {}", key, val));
                    }
                }
                let dict = format!("{{{}}}", fields.join(", "));
                return format!("{}{}{}", before, dict, after);
            }
        }
        line.to_string()
    }
}

// JS: Size.from(x) -> x
fn simplify_js_size_from(line: &str) -> String {
    let needle = "Size.from(";
    if let Some(start) = line.find(needle) {
        // Find matching ')'
        let mut depth: i32 = 0;
        let mut end = start + needle.len();
        let chars: Vec<char> = line.chars().collect();
        let mut i = end;
        while i < chars.len() {
            let c = chars[i];
            if c == '(' { depth += 1; }
            if c == ')' {
                if depth == 0 { end = i; break; }
                depth -= 1;
            }
            i += 1;
        }
        if end > start + needle.len() {
            let before = &line[..start];
            let inner = &line[start + needle.len()..end];
            let after = &line[end + 1..];
            return format!("{}{}{}", before, inner.trim(), after);
        }
    }
    line.to_string()
}

// JS: SamplerOptions { repeat_x: true, ... } -> { repeat_x: true, ... }
fn js_objectize_sampleroptions_literal(line: &str) -> String {
    let needle = "SamplerOptions {";
    if let Some(start) = line.find(needle) {
        let chars: Vec<char> = line.chars().collect();
        let mut depth = 0i32;
        let mut i = start + needle.len();
        let mut end = i;
        while i < chars.len() {
            let c = chars[i];
            if c == '{' { depth += 1; }
            if c == '}' {
                if depth == 0 { end = i; break; }
                depth -= 1;
            }
            i += 1;
        }
        if end > start {
            let before = &line[..start];
            let inside = &line[start + needle.len()..end];
            let after = &line[end + 1..];
            let mut fields: Vec<String> = Vec::new();
            for part in inside.split(',') {
                let p = part.trim();
                if p.is_empty() { continue; }
                if let Some(colon) = p.find(':') {
                    let key = p[..colon].trim();
                    let mut val = p[colon + 1..].trim().to_string();
                    // Map booleans and None
                    val = val.replace("None", "null");
                    // keep true/false as-is for JS
                    fields.push(format!("{}: {}", key, val));
                }
            }
            return format!("{}{{{}}}{}", before, fields.join(", "), after);
        }
    }
    line.to_string()
}

// JS: Region.new(a,b,c,d) -> [a,b,c,d]
fn js_region_new_to_array(line: &str) -> String {
    let needle = "Region.new(";
    if let Some(start) = line.find(needle) {
        let chars: Vec<char> = line.chars().collect();
        let mut depth = 0i32;
        let mut i = start + needle.len();
        let mut end = i;
        while i < chars.len() {
            let c = chars[i];
            if c == '(' { depth += 1; }
            if c == ')' {
                if depth == 0 { end = i; break; }
                depth -= 1;
            }
            i += 1;
        }
        if end > start {
            let before = &line[..start];
            let args = &line[start + needle.len()..end];
            let after = &line[end + 1..];
            return format!("{}[{}]{}", before, args.trim(), after);
        }
    }
    line.to_string()
}
