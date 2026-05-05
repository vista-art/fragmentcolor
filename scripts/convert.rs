mod convert {
    use crate::{js_objectize_sampleroptions_literal, js_region_new_to_array, simplify_js_size_from};

    /// Target languages for the Rust→FFI example transpiler.
    ///
    /// All four variants are first-class entries to the same `convert(...)`
    /// pipeline. During the per-line emit pass Swift and Kotlin share JS's
    /// control-flow shape (`{ ... }` blocks, `;` terminators, the
    /// `new Type(...)` constructor form), so internal match sites group
    /// them together via `Lang::Js | Lang::Swift | Lang::Kotlin =>` arms.
    /// The bulk-text post-processors in `scripts/{swift,kotlin}.rs` then
    /// finish the job — backticks → `"""`, `null` → `nil`, throwing-init
    /// `try!` insertion, etc. Folding those post-processors into per-line
    /// match arms here is the next refactor; this enum is honest about
    /// the four supported targets in the meantime.
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    enum Lang {
        Js,
        Py,
        Swift,
        Kotlin,
    }

    pub fn to_js(items: &[(String, bool)]) -> String {
        // Web JS output: collapse multi-line statements and unwrap Rust
        // 1-tuple-wrapped call args (`f((a, b))` → `f(a, b)`). The Swift
        // and Kotlin emitters skip both peephole passes because their own
        // tuple-aware rewrites (`kotlin::rewrite_texturemipchain_prepare_tuple`,
        // `kotlin::rewrite_shader_new_to_compose`, swift's tuple→Size
        // baseSize rewrite) rely on the original Rust shape.
        let raw = convert(items, Lang::Js, /*aggressive_js_flatten=*/ true);
        flatten_tuple_call_args_js_multiline(&raw)
    }

    pub fn to_py(items: &[(String, bool)]) -> String {
        convert(items, Lang::Py, false)
    }

    /// Transpile a Rust example into idiomatic Swift for the website tabs
    /// and the `platforms/swift/examples/` healthcheck inputs.
    ///
    /// Two-phase pipeline: the per-line emitter (`convert(_, Lang::Swift, _)`)
    /// produces a JS-shaped intermediate (Swift shares JS's control flow),
    /// then `swift::finalize` finishes the syntactic swaps that don't
    /// fit a per-line model — backtick template literals → `"""`,
    /// `null` → `nil`, `try` insertion for throwing inits, the canvas →
    /// `createTextureTarget` rewrite for headless examples, etc.
    pub fn to_swift(items: &[(String, bool)]) -> String {
        let ir = convert(items, Lang::Swift, /*aggressive_js_flatten=*/ false);
        super::swift::finalize(&ir)
    }

    /// Transpile a Rust example into idiomatic Kotlin.
    ///
    /// Same two-phase pipeline as `to_swift`: per-line emit
    /// (`convert(_, Lang::Kotlin, _)`) yields a JS-shaped intermediate,
    /// then `kotlin::finalize` applies the bulk-text rewrites
    /// (`const` → `val`, drop `new`, drop trailing `;`, backticks →
    /// `"""`, `await` prefix drop because `suspend` functions are called
    /// directly inside coroutine scope).
    pub fn to_kotlin(items: &[(String, bool)]) -> String {
        let ir = convert(items, Lang::Kotlin, /*aggressive_js_flatten=*/ false);
        super::kotlin::finalize(&ir)
    }

    /// Multi-line variant of `flatten_tuple_call_args_js`. Splits the
    /// joined output by lines and applies the per-line flatten so calls
    /// that survived as single logical lines (post
    /// `reassemble_open_bracket_lines`) still have their wrap stripped.
    /// Preserves a trailing newline if the input had one.
    fn flatten_tuple_call_args_js_multiline(s: &str) -> String {
        let trailing_newline = s.ends_with('\n');
        let mut out = s
            .lines()
            .map(flatten_tuple_call_args_js)
            .collect::<Vec<_>>()
            .join("\n");
        if trailing_newline {
            out.push('\n');
        }
        out
    }

    fn is_ident_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }

    // Drop Rust integer / float type suffixes from numeric literals:
    //   `0u8` → `0`, `64u32` → `64`, `100.0_f32` → `100.0`, `1isize` → `1`.
    // Walks the string, finds digit runs (with optional `.` for floats),
    // and strips a trailing suffix if it matches one of the known forms.
    // Skips matches that don't follow a real numeric token, so identifiers
    // like `vec3<f32>` or `usize_helper` are left untouched (the suffix
    // probe only fires after a digit).
    fn strip_rust_numeric_suffixes(line: &str) -> String {
        let chars: Vec<char> = line.chars().collect();
        let mut out = String::with_capacity(line.len());
        let mut i = 0usize;
        while i < chars.len() {
            // Don't engage mid-identifier: a digit immediately after an
            // identifier char is part of the identifier, not a literal.
            let prev_is_ident = i > 0 && is_ident_char(chars[i - 1]) && !chars[i - 1].is_ascii_digit();
            if !prev_is_ident && chars[i].is_ascii_digit() {
                // Scan the numeric token — digits, underscores, hex prefix,
                // and at most one decimal point. Hex (`0x...`) consumes
                // hex digits.
                let start = i;
                let mut is_hex = false;
                if chars[i] == '0' && i + 1 < chars.len() && (chars[i + 1] == 'x' || chars[i + 1] == 'X') {
                    out.push(chars[i]);
                    out.push(chars[i + 1]);
                    i += 2;
                    is_hex = true;
                    while i < chars.len() && (chars[i].is_ascii_hexdigit() || chars[i] == '_') {
                        out.push(chars[i]);
                        i += 1;
                    }
                } else {
                    let mut saw_dot = false;
                    while i < chars.len()
                        && (chars[i].is_ascii_digit()
                            || chars[i] == '_'
                            || (chars[i] == '.' && !saw_dot
                                && i + 1 < chars.len()
                                && chars[i + 1].is_ascii_digit()))
                    {
                        if chars[i] == '.' {
                            saw_dot = true;
                        }
                        out.push(chars[i]);
                        i += 1;
                    }
                }
                let _ = (start, is_hex);
                // Optional `_` separator before suffix
                let mut probe = i;
                if probe < chars.len() && chars[probe] == '_' {
                    probe += 1;
                }
                // Match suffix
                let suffix_chars: Vec<char> = chars.get(probe..).unwrap_or(&[]).to_vec();
                let suffix_str: String = suffix_chars.iter().collect();
                let candidates: &[&str] = &[
                    "usize", "isize",
                    "u128", "i128",
                    "u64", "i64", "u32", "i32", "u16", "i16", "u8", "i8",
                    "f64", "f32",
                ];
                let mut consumed = None;
                for cand in candidates {
                    if suffix_str.starts_with(cand) {
                        let after = probe + cand.len();
                        if after >= chars.len() || !is_ident_char(chars[after]) {
                            consumed = Some(after);
                            break;
                        }
                    }
                }
                if let Some(after) = consumed {
                    i = after;
                }
                continue;
            }
            out.push(chars[i]);
            i += 1;
        }
        out
    }

    // Merge multi-line statements whose first line opens unbalanced
    // brackets / parens / braces. Walks line-by-line, tracking depth of
    // `()`, `[]`, `{}` and Rust raw-string state. While depth > 0 at the
    // end of a line, the next line is concatenated with a single space
    // separator. This collapses multi-arg method calls split across
    // lines (e.g. `f(\n    a,\n    b,\n)`) into a single logical line so
    // the downstream peephole transforms can see the whole expression.
    fn reassemble_open_bracket_lines(lines: Vec<String>) -> Vec<String> {
        let mut out: Vec<String> = Vec::with_capacity(lines.len());
        let mut buffer: Option<String> = None;
        let mut depth_paren: i32 = 0;
        let mut depth_brack: i32 = 0;
        let mut depth_brace: i32 = 0;
        // Per-line walker: counts unbalanced delimiters, tracks raw
        // string state, and reports whether the line ends inside a
        // line comment. `raw_hashes_in` carries the cross-line
        // raw-string state. Returns (delta_paren, delta_brack,
        // delta_brace, raw_hashes_out, has_line_comment).
        fn line_deltas(
            line: &str,
            raw_hashes_in: Option<usize>,
        ) -> (i32, i32, i32, Option<usize>, bool) {
            let chars: Vec<char> = line.chars().collect();
            let mut in_str: Option<char> = None;
            let mut in_line_comment = false;
            let mut local_paren: i32 = 0;
            let mut local_brack: i32 = 0;
            let mut local_brace: i32 = 0;
            let mut k = 0usize;
            let mut local_raw_hashes: Option<usize> = raw_hashes_in;
            while k < chars.len() {
                let c = chars[k];
                if in_line_comment {
                    break;
                }
                if local_raw_hashes.is_some() {
                    if c == '"' {
                        let need = local_raw_hashes.unwrap();
                        let mut got = 0usize;
                        let mut j = k + 1;
                        while j < chars.len() && chars[j] == '#' && got < need {
                            got += 1;
                            j += 1;
                        }
                        if got == need {
                            local_raw_hashes = None;
                            k = j;
                            continue;
                        }
                    }
                    k += 1;
                    continue;
                }
                if let Some(q) = in_str {
                    if c == '\\' && k + 1 < chars.len() {
                        k += 2;
                        continue;
                    }
                    if c == q {
                        in_str = None;
                    }
                    k += 1;
                    continue;
                }
                // Detect raw string opener `r#*"`.
                if c == 'r' {
                    let mut j = k + 1;
                    let mut hashes = 0usize;
                    while j < chars.len() && chars[j] == '#' {
                        hashes += 1;
                        j += 1;
                    }
                    if j < chars.len() && chars[j] == '"' {
                        local_raw_hashes = Some(hashes);
                        k = j + 1;
                        continue;
                    }
                }
                match c {
                    '"' | '\'' | '`' => {
                        in_str = Some(c);
                    }
                    '/' if k + 1 < chars.len() && chars[k + 1] == '/' => {
                        in_line_comment = true;
                    }
                    '(' => local_paren += 1,
                    ')' => local_paren -= 1,
                    '[' => local_brack += 1,
                    ']' => local_brack -= 1,
                    '{' => local_brace += 1,
                    '}' => local_brace -= 1,
                    _ => {}
                }
                k += 1;
            }
            (
                local_paren,
                local_brack,
                local_brace,
                local_raw_hashes,
                in_line_comment,
            )
        }
        let mut raw_hashes: Option<usize> = None;
        // Whether the line we just buffered ended inside a `//` line
        // comment. If so, the next line MUST be appended with a newline
        // — collapsing to a space would slurp the next code into the
        // comment.
        let mut prev_had_line_comment = false;
        for line in lines {
            let (dp, db, dbr, new_raw, has_comment) = line_deltas(&line, raw_hashes);
            depth_paren += dp;
            depth_brack += db;
            depth_brace += dbr;
            // Append to buffer or start a new buffer.
            if let Some(buf) = buffer.as_mut() {
                // Inside a raw string we use `\n` to preserve WGSL line
                // structure; if the previous fragment ended inside a `//`
                // comment, also use `\n` so the comment terminates before
                // the next token. Otherwise collapse to a single space so
                // multi-arg calls fit on one logical line.
                if raw_hashes.is_some() || prev_had_line_comment {
                    buf.push('\n');
                    buf.push_str(&line);
                } else {
                    buf.push(' ');
                    buf.push_str(line.trim_start());
                }
            } else if depth_paren > 0 || depth_brack > 0 || depth_brace > 0 || new_raw.is_some()
            {
                buffer = Some(line.clone());
            } else {
                out.push(line.clone());
            }
            raw_hashes = new_raw;
            prev_had_line_comment = has_comment;
            // Flush when balanced AND no open raw string.
            if depth_paren <= 0
                && depth_brack <= 0
                && depth_brace <= 0
                && raw_hashes.is_none()
                && let Some(buf) = buffer.take()
            {
                out.push(buf);
                depth_paren = depth_paren.max(0);
                depth_brack = depth_brack.max(0);
                depth_brace = depth_brace.max(0);
            }
        }
        if let Some(buf) = buffer.take() {
            out.push(buf);
        }
        out
    }

    // Merge continuation lines (those starting with `.`) into the
    // previous statement so multi-line method chains read as one logical
    // line. Tracks `r#"..."#` raw-string boundaries so WGSL / shader
    // bodies inside raw strings are left untouched.
    fn reassemble_chain_lines(lines: Vec<String>) -> Vec<String> {
        let mut out: Vec<String> = Vec::with_capacity(lines.len());
        let mut raw_hashes: Option<usize> = None;
        for line in lines {
            let starts_dot = raw_hashes.is_none() && {
                let t = line.trim_start();
                t.starts_with('.') && !t.starts_with("..")
            };
            let prev_open = match out.last() {
                None => false,
                Some(prev) => {
                    let trimmed = prev.trim_end();
                    !trimmed.ends_with(';') && !trimmed.ends_with('{') && !trimmed.is_empty()
                }
            };
            if starts_dot && prev_open {
                let prev = out.pop().unwrap();
                let merged = format!("{}{}", prev.trim_end(), line.trim_start());
                out.push(merged);
            } else {
                out.push(line.clone());
            }
            raw_hashes = update_raw_string_state(raw_hashes, &out.last().cloned().unwrap_or_default());
        }
        out
    }

    // Track Rust raw-string state across a single line. Returns the new
    // hash count (Some(n) = inside `r#*"..."#*` with `n` hashes) or None
    // (outside any raw string).
    fn update_raw_string_state(start: Option<usize>, line: &str) -> Option<usize> {
        let chars: Vec<char> = line.chars().collect();
        let mut state = start;
        let mut i = 0usize;
        while i < chars.len() {
            match state {
                None => {
                    if chars[i] == 'r' {
                        let mut j = i + 1;
                        let mut hashes = 0usize;
                        while j < chars.len() && chars[j] == '#' {
                            hashes += 1;
                            j += 1;
                        }
                        if j < chars.len() && chars[j] == '"' {
                            state = Some(hashes);
                            i = j + 1;
                            continue;
                        }
                    }
                    i += 1;
                }
                Some(n) => {
                    if chars[i] == '"' {
                        let mut k = i + 1;
                        let mut got = 0usize;
                        while k < chars.len() && chars[k] == '#' && got < n {
                            got += 1;
                            k += 1;
                        }
                        if got == n {
                            state = None;
                            i = k;
                            continue;
                        }
                    }
                    i += 1;
                }
            }
        }
        state
    }

    // Convert Rust array-repeat literal `[expr; N]` (e.g. `[0u8; 256]`,
    // already suffix-stripped to `[0; 256]` by the pre-pass) into a
    // valid per-language form. Walks bracket-and-paren balanced so
    // nested expressions don't trip the pattern.
    //   JS: `Array(N).fill(expr)` — picked up + idiomatized further by
    //       `swift::finalize` / `kotlin::finalize`.
    //   Py: `[expr] * N`.
    fn convert_rust_array_repeat(line: &str, lang: Lang) -> String {
        if !line.contains('[') || !line.contains(';') {
            return line.to_string();
        }
        let chars: Vec<char> = line.chars().collect();
        let mut out = String::with_capacity(line.len());
        let mut i = 0usize;
        while i < chars.len() {
            if chars[i] == '[' {
                // Walk to matching `]`, recording first `;` at depth 0.
                let mut depth_brack = 0i32;
                let mut depth_paren = 0i32;
                let mut semi_pos: Option<usize> = None;
                let mut close_pos: Option<usize> = None;
                let mut k = i + 1;
                while k < chars.len() {
                    match chars[k] {
                        '[' => depth_brack += 1,
                        ']' => {
                            if depth_brack == 0 {
                                close_pos = Some(k);
                                break;
                            }
                            depth_brack -= 1;
                        }
                        '(' => depth_paren += 1,
                        ')' => depth_paren -= 1,
                        ';' if depth_brack == 0 && depth_paren == 0 && semi_pos.is_none() => {
                            semi_pos = Some(k);
                        }
                        _ => {}
                    }
                    k += 1;
                }
                if let (Some(semi), Some(close)) = (semi_pos, close_pos) {
                    let expr: String = chars[i + 1..semi].iter().collect();
                    let count: String = chars[semi + 1..close].iter().collect();
                    let expr = expr.trim();
                    let count = count.trim();
                    let replacement = match lang {
                        Lang::Js | Lang::Swift | Lang::Kotlin => format!("Array({}).fill({})", count, expr),
                        Lang::Py => format!("[{}] * ({})", expr, count),
                    };
                    out.push_str(&replacement);
                    i = close + 1;
                    continue;
                }
            }
            out.push(chars[i]);
            i += 1;
        }
        out
    }

    // Drop Rust unary deref `*var` when `*` sits in expression-start
    // position (preceded by `(`, `,`, ` `, `=`, etc.) and is followed by
    // an identifier. Multiplication (`a * b`) and pointer types are not
    // matched because both sides would be ident chars.
    fn strip_rust_deref_star(line: &str) -> String {
        let chars: Vec<char> = line.chars().collect();
        let mut out = String::with_capacity(line.len());
        let mut i = 0usize;
        while i < chars.len() {
            if chars[i] == '*' {
                let prev = if i == 0 {
                    None
                } else {
                    Some(chars[i - 1])
                };
                let next = chars.get(i + 1).copied();
                let prev_is_expr_boundary = match prev {
                    None => true,
                    Some(c) => matches!(c, '(' | ',' | '=' | '!' | '&' | '|' | ':' | '?' | '<' | '>')
                        || c.is_whitespace(),
                };
                let next_is_ident_start = matches!(next, Some(c) if c.is_ascii_alphabetic() || c == '_');
                if prev_is_expr_boundary && next_is_ident_start {
                    // Skip the `*`
                    i += 1;
                    continue;
                }
            }
            out.push(chars[i]);
            i += 1;
        }
        out
    }

    // Helper: convert common Rust Vec constructs to JS/Python arrays
    fn convert_vec_syntax(s: &str) -> String {
        let mut out = s.replace("Vec::new()", "[]");
        out = out.replace("vec![", "[");
        out = out.replace("vec ![", "["); // tolerate space
        out
    }

    // Strip Rust slice-cast method calls that have no JS / Python analogue:
    // `bytes.as_slice()`, `vec.as_ref()`, `s.to_vec()`. These coerce types
    // in Rust but the receiver in JS / Python already has the right shape.
    // Only matches an empty argument list (`(...)` with whitespace only).
    fn strip_rust_coercion_calls(line: &str) -> String {
        let names = [".as_slice", ".as_ref", ".to_vec"];
        let mut out = line.to_string();
        for needle in names {
            while let Some(pos) = out.find(needle) {
                let chars: Vec<char> = out.chars().collect();
                let after_name = pos + needle.len();
                let mut k = after_name;
                while k < chars.len() && chars[k].is_whitespace() {
                    k += 1;
                }
                if k >= chars.len() || chars[k] != '(' {
                    break;
                }
                k += 1;
                while k < chars.len() && chars[k].is_whitespace() {
                    k += 1;
                }
                if k >= chars.len() || chars[k] != ')' {
                    break;
                }
                let close = k;
                let next_ch = chars.get(close + 1).copied();
                if let Some(c) = next_ch
                    && is_ident_char(c)
                {
                    break;
                }
                let mut new_out = String::with_capacity(out.len());
                new_out.push_str(&chars[..pos].iter().collect::<String>());
                new_out.push_str(&chars[close + 1..].iter().collect::<String>());
                out = new_out;
            }
        }
        out
    }

    // Detect a Rust 1-tuple-wrapped argument list and unwrap it for JS.
    //
    // `f((a, b, c))` becomes `f(a, b, c)`. Rust uses tuples to dispatch
    // through `From<(A, B, ...)>` impls; JS exposes the same shapes as
    // plain positional args. Only fires when the entire argument list of
    // the call is a single parenthesised tuple — `f([a, b])`, `f((a))`
    // (parens around one expr, not a tuple), and `f(x, (a, b))` are left
    // alone.
    fn flatten_tuple_call_args_js(line: &str) -> String {
        let chars: Vec<char> = line.chars().collect();
        let mut out = String::with_capacity(line.len());
        let mut i = 0usize;
        while i < chars.len() {
            if chars[i] != '(' {
                out.push(chars[i]);
                i += 1;
                continue;
            }
            let prev_is_ident = i > 0 && is_ident_char(chars[i - 1]);
            if !prev_is_ident {
                out.push(chars[i]);
                i += 1;
                continue;
            }
            let mut depth_paren = 1i32;
            let mut depth_brack = 0i32;
            let mut depth_brace = 0i32;
            let mut in_str: Option<char> = None;
            let mut j = i + 1;
            let mut close_outer: Option<usize> = None;
            while j < chars.len() {
                let c = chars[j];
                match in_str {
                    Some(q) => {
                        if c == '\\' && j + 1 < chars.len() {
                            j += 2;
                            continue;
                        }
                        if c == q {
                            in_str = None;
                        }
                    }
                    None => match c {
                        '"' | '\'' | '`' => in_str = Some(c),
                        '(' => depth_paren += 1,
                        ')' => {
                            depth_paren -= 1;
                            if depth_paren == 0 && depth_brack == 0 && depth_brace == 0 {
                                close_outer = Some(j);
                                break;
                            }
                        }
                        '[' => depth_brack += 1,
                        ']' => depth_brack -= 1,
                        '{' => depth_brace += 1,
                        '}' => depth_brace -= 1,
                        _ => {}
                    },
                }
                j += 1;
            }
            let Some(close_outer) = close_outer else {
                out.push_str(&chars[i..].iter().collect::<String>());
                break;
            };
            let inner_start = i + 1;
            let inner_end = close_outer;
            let mut a = inner_start;
            while a < inner_end && chars[a].is_whitespace() {
                a += 1;
            }
            let mut b = inner_end;
            while b > a && chars[b - 1].is_whitespace() {
                b -= 1;
            }
            let is_paren_wrapped = a < b && chars[a] == '(' && chars[b - 1] == ')';
            if !is_paren_wrapped {
                out.push(chars[i]);
                i += 1;
                continue;
            }
            let mut dp = 1i32;
            let mut db = 0i32;
            let mut dbr = 0i32;
            let mut s2: Option<char> = None;
            let mut k = a + 1;
            let mut matched: Option<usize> = None;
            while k < b {
                let c = chars[k];
                match s2 {
                    Some(q) => {
                        if c == '\\' && k + 1 < b {
                            k += 2;
                            continue;
                        }
                        if c == q {
                            s2 = None;
                        }
                    }
                    None => match c {
                        '"' | '\'' | '`' => s2 = Some(c),
                        '(' => dp += 1,
                        ')' => {
                            dp -= 1;
                            if dp == 0 && db == 0 && dbr == 0 {
                                matched = Some(k);
                                break;
                            }
                        }
                        '[' => db += 1,
                        ']' => db -= 1,
                        '{' => dbr += 1,
                        '}' => dbr -= 1,
                        _ => {}
                    },
                }
                k += 1;
            }
            let Some(matched) = matched else {
                out.push(chars[i]);
                i += 1;
                continue;
            };
            if matched + 1 != b {
                out.push(chars[i]);
                i += 1;
                continue;
            }
            let mut has_top_comma = false;
            let mut dp2 = 0i32;
            let mut db2 = 0i32;
            let mut dbr2 = 0i32;
            let mut s3: Option<char> = None;
            let mut t_i = a + 1;
            while t_i < matched {
                let c = chars[t_i];
                match s3 {
                    Some(q) => {
                        if c == '\\' && t_i + 1 < matched {
                            t_i += 2;
                            continue;
                        }
                        if c == q {
                            s3 = None;
                        }
                    }
                    None => match c {
                        '"' | '\'' | '`' => s3 = Some(c),
                        '(' => dp2 += 1,
                        ')' => dp2 -= 1,
                        '[' => db2 += 1,
                        ']' => db2 -= 1,
                        '{' => dbr2 += 1,
                        '}' => dbr2 -= 1,
                        ',' if dp2 == 0 && db2 == 0 && dbr2 == 0 => {
                            has_top_comma = true;
                            break;
                        }
                        _ => {}
                    },
                }
                t_i += 1;
            }
            if !has_top_comma {
                out.push(chars[i]);
                i += 1;
                continue;
            }
            let inner: String = chars[a + 1..matched].iter().collect();
            out.push('(');
            out.push_str(inner.trim());
            out.push(')');
            i = close_outer + 1;
        }
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
        // Replace patterns like Type::method( -> Type.method(.
        //
        // Iterate by `chars()` rather than `as_bytes()` so multi-byte UTF-8
        // sequences (em-dashes, smart quotes, etc.) survive intact. The old
        // byte-loop pushed each raw byte as a `char`, which split the 3-byte
        // U+2014 (—) into U+00E2 + U+0080 + U+0094 — visible as the
        // mojibake "â" + 2 invisible controls in py/swift/kotlin output.
        let chars: Vec<char> = line.chars().collect();
        let mut out = String::with_capacity(line.len());
        let mut i = 0usize;
        while i < chars.len() {
            if i + 1 < chars.len() && chars[i] == ':' && chars[i + 1] == ':' {
                out.push('.');
                i += 2;
                continue;
            }
            out.push(chars[i]);
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
        // Handle `.await?` and bare `.await`:
        //   JS:  `expr.await(?)` → `await expr` (with the `?` separately
        //        triggering Rust error-propagation cleanup below).
        //   Py:  drop `.await` entirely — Python wrappers are sync.
        //
        // The Swift / Kotlin paths consume the JS output as IR and then
        // adjust: `swift::finalize` rewrites `await expr` → `try await expr`,
        // while `kotlin::finalize` drops the `await` prefix because uniffi
        // exposes async Rust as Kotlin `suspend` (no await needed).
        let mut out = line.to_string();
        // Process the `?`-suffixed form first so its tail (anything after
        // `.await?`) survives the rewrite intact.
        for needle in [".await?", ".await"] {
            if !out.contains(needle) {
                continue;
            }
            match lang {
                Lang::Js | Lang::Swift | Lang::Kotlin => {
                    if let Some(pos) = out.find(needle) {
                        // Preserve left side if present
                        if let Some(eq) = out[..pos].rfind('=') {
                            let (lhs, expr) = out.split_at(eq + 1);
                            let expr = expr.trim();
                            let before_await = &expr[..expr.rfind(needle).unwrap_or(expr.len())];
                            let mut s = String::new();
                            s.push_str(lhs);
                            s.push(' ');
                            s.push_str("await ");
                            s.push_str(before_await.trim());
                            let tail = &out[pos + needle.len()..];
                            s.push_str(tail);
                            out = s;
                        } else {
                            let before_await = &out[..pos];
                            let mut s = String::new();
                            s.push_str("await ");
                            s.push_str(before_await.trim());
                            let tail = &out[pos + needle.len()..];
                            s.push_str(tail);
                            out = s;
                        }
                    }
                }
                Lang::Py => {
                    out = out.replace(needle, "");
                }
            }
        }
        // Remove stray Rust error-propagation '?' for both langs (JS/Py)
        match lang {
            Lang::Js | Lang::Swift | Lang::Kotlin => {
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
            Lang::Js | Lang::Swift | Lang::Kotlin => Some(format!(
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
                Lang::Js | Lang::Swift | Lang::Kotlin => {
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
                        Lang::Js | Lang::Swift | Lang::Kotlin => format!("new {}({})", ty, args.trim()),
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
                    Lang::Js | Lang::Swift | Lang::Kotlin => {
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
        if matches!(lang, Lang::Js | Lang::Swift | Lang::Kotlin) {
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
            // 4) ScreenRegion.new(a,b,c,d) -> [a,b,c,d]
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
            Lang::Js | Lang::Swift | Lang::Kotlin => {
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
            Lang::Js | Lang::Swift | Lang::Kotlin => apply_renames_py(&rhs, js_renames),
        };

        let mut line_out = match lang {
            Lang::Js | Lang::Swift | Lang::Kotlin => ensure_js_semicolon(&format!("const {} = {}", var_out, rhs)),
            Lang::Py => format!("{} = {}", var_out, rhs),
        };
        // Python: Convert JS-style '//' comments to Python '#', but only when
        // the '//' is outside string and char literals (otherwise URLs like
        // "https://..." get mangled).
        if let Lang::Py = lang
            && let Some(idx) = find_comment_marker(&line_out)
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

    /// Find the first `//` occurrence outside string and char literals.
    /// Returns None if no comment marker exists or all occurrences are inside literals.
    fn find_comment_marker(s: &str) -> Option<usize> {
        let bytes = s.as_bytes();
        let mut i = 0;
        let mut in_string = false;
        let mut in_char = false;
        let mut escape = false;
        while i + 1 < bytes.len() {
            let c = bytes[i];
            if escape {
                escape = false;
                i += 1;
                continue;
            }
            if (in_string || in_char) && c == b'\\' {
                escape = true;
                i += 1;
                continue;
            }
            if c == b'"' && !in_char {
                in_string = !in_string;
            } else if c == b'\'' && !in_string {
                in_char = !in_char;
            } else if !in_string && !in_char && c == b'/' && bytes[i + 1] == b'/' {
                return Some(i);
            }
            i += 1;
        }
        None
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

    // Detect and convert a multi-line Rust raw string assignment into a
    // single emitted line. Two shapes are recognised:
    //   - `let var = Type::new(r#"..."#)` — constructor call (JS: `new Type`,
    //     Py: bare `Type`).
    //   - `let var = r#"..."#`             — bare string assignment (no
    //     constructor — emit the literal directly).
    // Returns (mapped_text, next_index) when a block is consumed, or None
    // if the line is not a let-with-raw-string at all.
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
        // Either `<Path>::new(r#"...` (constructor) or `r#"...` (bare).
        let (ty_opt, after_new): (Option<String>, &str) =
            if let Some(pos_new) = rhs_line.find("::new(") {
                let type_path = rhs_line[..pos_new].trim();
                let ty = type_path
                    .rsplit("::")
                    .next()
                    .unwrap_or(type_path)
                    .to_string();
                (Some(ty), &rhs_line[pos_new + "::new(".len()..])
            } else if rhs_line.trim_start().starts_with('r') {
                (None, rhs_line)
            } else {
                return None;
            };

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
                let mapped = render_raw_string_assignment(
                    lang,
                    &var_out,
                    ty_opt.as_deref(),
                    &body_lines,
                );
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
                let mapped = render_raw_string_assignment(
                    lang,
                    &var_out,
                    ty_opt.as_deref(),
                    &body_lines,
                );
                return Some((mapped, j + 1));
            } else {
                body_lines.push(l.clone());
                j += 1;
            }
        }
        // If we get here, we didn't find a proper terminator; don't transform.
        None
    }

    /// Emit either `let var = new Ty(\`body\`)` (constructor case) or
    /// `let var = \`body\`` (bare-string case) per language. JS uses
    /// backticks (template literal) so swift.rs/kotlin.rs can swap them
    /// for triple-quoted strings; Python uses triple-quoted directly.
    fn render_raw_string_assignment(
        lang: Lang,
        var_out: &str,
        ty: Option<&str>,
        body_lines: &[String],
    ) -> String {
        let body = body_lines.join("\n");
        match (lang, ty) {
            (Lang::Js | Lang::Swift | Lang::Kotlin, Some(ty)) => format!("const {} = new {}(`\n{}\n`);", var_out, ty, body),
            (Lang::Js | Lang::Swift | Lang::Kotlin, None) => format!("const {} = `\n{}\n`;", var_out, body),
            (Lang::Py, Some(ty)) => format!("{} = {}(\"\"\"\n{}\n\"\"\")", var_out, ty, body),
            (Lang::Py, None) => format!("{} = \"\"\"\n{}\n\"\"\"", var_out, body),
        }
    }

    fn convert(items: &[(String, bool)], lang: Lang, aggressive_js_flatten: bool) -> String {
        use std::collections::HashMap;
        // Collect visible lines only
        let mut src: Vec<String> = Vec::new();
        for (t, hidden) in items {
            if !*hidden {
                src.push(t.clone());
            }
        }
        // Pre-pass: strip Rust idioms with no JS / Python / Swift / Kotlin
        // analogue. These transformations are language-agnostic — they
        // make the line "look like" idiomatic non-Rust code so the
        // downstream branches don't have to special-case Rust integer
        // literal suffixes (`0u8`, `64u32`) or unary deref (`*var.id()`).
        for line in src.iter_mut() {
            *line = strip_rust_numeric_suffixes(line);
            *line = strip_rust_deref_star(line);
            // Rust slice-cast no-ops (`bytes.as_slice()`, `vec.as_ref()`,
            // `s.to_vec()`) have no JS / Python / Swift / Kotlin analogue —
            // the receiver already has the right shape.
            *line = strip_rust_coercion_calls(line);
            // `vec![...]` → `[...]`. Strip BEFORE the array-repeat pass
            // so `vec![0; N]` → `[0; N]` → `Array(N).fill(0)`. Otherwise
            // the array-repeat pass leaves the `vec!` prefix orphaned.
            *line = convert_vec_syntax(line);
            *line = convert_rust_array_repeat(line, lang);
        }
        // Pre-pass: collapse multi-line statements whose first line
        // opens unbalanced parens / brackets / braces. Without this, a
        // statement like
        //   `f(\n    a,\n    b,\n)`
        // splits into 4 lines and the line-by-line transforms below can't
        // peephole the whole expression (e.g. the tuple-flatten pass for
        // `f((a, b))` wouldn't fire). Raw-string–aware so WGSL bodies are
        // left untouched.
        //
        // Gated behind `aggressive_js_flatten` because the Swift / Kotlin
        // emitters call `convert(_, _, /*aggressive_js_flatten=*/ false)`
        // and rely on the original line layout (e.g.
        // `Shader.new([\n    "...",\n])` is detected by
        // `kotlin::rewrite_shader_new_to_compose` only after `kotlin::finalize`
        // does its own array→arrayOf substitution; if we join here, the
        // array elements end up on a single line with line comments
        // slurping subsequent items).
        if aggressive_js_flatten {
            src = reassemble_open_bracket_lines(src);
        }
        // Pre-pass: reassemble multi-line method chains. A line that begins
        // with `.` (after trimming) and follows an unterminated previous
        // line is the continuation of a chain (`expr\n  .method()\n  .await?;`).
        // The downstream line-by-line transforms can't see across lines,
        // so we collapse the chain into one logical line first. Skip this
        // for lines inside Rust raw strings (`r#"..."#`) — those contain
        // arbitrary content (typically WGSL) that must not be merged.
        src = reassemble_chain_lines(src);
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
                // Drop names that aren't exposed as constructible types in the
                // foreign bindings (Target/WindowTarget/TextureTarget — the
                // first is a Rust trait, the latter two are returned by
                // factory methods, never constructed; Size is auto-converted
                // from arrays/tuples; VertexValue is internal).
                // SamplerOptions IS exposed (uniffi::Record / wasm_bindgen /
                // pyo3::pyclass) so it stays in the import list.
                let list: Vec<String> = list
                    .into_iter()
                    .filter(|name| {
                        name != "Target"
                            && name != "WindowTarget"
                            && name != "TextureTarget"
                            && name != "Size"
                            && name != "VertexValue"
                    })
                    .collect();
                match lang {
                    Lang::Js | Lang::Swift | Lang::Kotlin => out.push(format!(
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
                    Lang::Js | Lang::Swift | Lang::Kotlin => ensure_js_semicolon(&mapped),
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
            if matches!(lang, Lang::Js | Lang::Swift | Lang::Kotlin) {
                // None -> null
                line = line.replace("None", "null");
                // SamplerOptions { ... } -> { ... }
                if line.contains("SamplerOptions {") {
                    line = js_objectize_sampleroptions_literal(&line);
                }
                // Size.from(x) -> x
                line = simplify_js_size_from(&line);
                // ScreenRegion.new(a,b,c,d) -> [a,b,c,d]
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

            // 2) Drop explicit module prefix on every target — Swift/Kotlin
            // both fall through `swift::finalize` / `kotlin::finalize` after this so
            // they want the same plain `Shader` token the JS/Py emitters do.
            line = line.replace("fragmentcolor.Shader", "Shader");

            // 3) Handling for Shader::default();
            if line.contains("Shader::default()")
                || line.contains("Shader.default()")
                || line.contains("fragmentcolor::Shader::default()")
                || line.contains("fragmentcolor::Shader.default()")
            {
                line = match lang {
                    Lang::Js | Lang::Swift | Lang::Kotlin => line
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
                // Convert JS-style '//' comments to Python '#', skipping '//'
                // inside string and char literals (URLs etc).
                if let Some(idx) = find_comment_marker(&line) {
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
            if matches!(lang, Lang::Js | Lang::Swift | Lang::Kotlin) {
                line = camelize_method_calls_js(&line);
            }

            // 8) Language-specific trailing cleanup
            match lang {
                Lang::Js | Lang::Swift | Lang::Kotlin => {
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
                Lang::Js | Lang::Swift | Lang::Kotlin => {
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

// JS: ScreenRegion.new(a,b,c,d) -> [a,b,c,d]
fn js_region_new_to_array(line: &str) -> String {
    let needle = "ScreenRegion.new(";
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
