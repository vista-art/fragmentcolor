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

    pub fn js_to_swift(js: &str) -> String {
        let js = swap_backticks_for_triple_quotes(js);

        let mut out: Vec<String> = Vec::with_capacity(js.lines().count());
        let mut has_fragmentcolor_import = false;

        for raw in js.lines() {
            let line = raw.to_string();
            let trimmed = line.trim_start();

            if trimmed.starts_with("import ") && trimmed.contains("from \"fragmentcolor\"") {
                if !has_fragmentcolor_import {
                    out.push("import FragmentColor".to_string());
                    has_fragmentcolor_import = true;
                }
                continue;
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
