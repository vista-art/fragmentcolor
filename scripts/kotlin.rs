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
