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

        let mut out: Vec<String> = Vec::with_capacity(js.lines().count());
        let mut has_fragmentcolor_import = false;

        for raw in js.lines() {
            let line = raw.to_string();
            let trimmed = line.trim_start();

            if trimmed.starts_with("import ") && trimmed.contains("from \"fragmentcolor\"") {
                if !has_fragmentcolor_import {
                    out.push("import org.fragmentcolor.*".to_string());
                    has_fragmentcolor_import = true;
                }
                continue;
            }

            out.push(rewrite_line(&line));
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
        out = rewrite_array_fill(&out);
        out = swap_single_quoted_strings(&out);
        out = rewrite_bracket_array_to_arrayof(&out);

        out
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

    /// `Array(N).fill(expr)` → `Array(N) { expr }` (Kotlin lambda init).
    /// Bracket-balanced; mirrors `swift::rewrite_array_fill`.
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
                            let expr: String = fill_arg_chars[..fc].iter().collect();
                            out.push_str(&format!(
                                "Array({}) {{ {} }}",
                                count.trim(),
                                expr.trim()
                            ));
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

    fn is_ident_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }
}
