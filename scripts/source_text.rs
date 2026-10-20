mod source_text {
    //! Text scanners the example transpilers share.
    //!
    //! They read hand-written extension files
    //! and the call sites the shared emitter writes,
    //! and none of them depends on the language of the text.

    use std::fs;
    use std::ops::Range;
    use std::path::PathBuf;

    /// Which brackets nest when [`arg_spans`] looks for top-level commas.
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub enum Nesting {
        /// `()`, `[]` and `{}`, as in a call.
        Call,
        /// Those and `<>`, as in a parameter list. The `>` of a `->` closes nothing.
        Types,
    }

    /// The files in `dir` whose names end in `suffix`, sorted.
    /// The build script reruns when one of them changes or `dir` gains a file.
    pub fn extension_files(dir: &str, suffix: &str) -> Vec<PathBuf> {
        println!("cargo::rerun-if-changed={dir}");
        let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("cannot list {dir}: {e}"));
        let mut paths: Vec<PathBuf> = entries
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.ends_with(suffix)))
            .collect();
        paths.sort();
        for path in &paths {
            println!("cargo::rerun-if-changed={}", path.display());
        }
        paths
    }

    /// `text` with comments and string literals blanked out,
    /// so braces and keywords inside them don't count. Byte offsets stay put.
    pub fn code_only(text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        let mut rest = text;
        while let Some(c) = rest.chars().next() {
            let skip = if rest.starts_with("//") {
                rest.find('\n').unwrap_or(rest.len())
            } else if rest.starts_with("/*") {
                rest.find("*/").map_or(rest.len(), |p| p + 2)
            } else if c == '"' {
                string_end(rest)
            } else {
                out.push(c);
                rest = &rest[c.len_utf8()..];
                continue;
            };
            for c in rest[..skip].chars() {
                let blank = if c == '\n' { '\n' } else { ' ' };
                out.extend(std::iter::repeat_n(blank, c.len_utf8()));
            }
            rest = &rest[skip..];
        }
        out
    }

    /// Byte length of the string literal `text` starts with.
    pub fn string_end(text: &str) -> usize {
        let mut escaped = false;
        for (k, c) in text.char_indices().skip(1) {
            match c {
                '\\' if !escaped => escaped = true,
                '"' if !escaped => return k + 1,
                _ => escaped = false,
            }
        }
        text.len()
    }

    /// Each of `words` that stands as a whole word at brace depth 0, with its byte offset.
    pub fn top_level_words<'a>(code: &str, words: &[&'a str]) -> Vec<(&'a str, usize)> {
        let mut found = Vec::new();
        let mut depth = 0i32;
        for (at, c) in code.char_indices() {
            match c {
                '{' => depth += 1,
                '}' => depth -= 1,
                _ if depth == 0 => {
                    let bounded = !code[..at].ends_with(|c: char| is_ident_char(c) || c == '.');
                    if let Some(word) = words.iter().find(|w| {
                        bounded
                            && code[at..].starts_with(**w)
                            && !code[at + w.len()..].starts_with(is_ident_char)
                    }) {
                        found.push((*word, at));
                    }
                }
                _ => {}
            }
        }
        found
    }

    pub fn skip_space(code: &str, at: usize) -> usize {
        code.len() - code[at..].trim_start().len()
    }

    /// The identifier starting at byte `at` and the offset just past it.
    pub fn read_ident(code: &str, at: usize) -> (&str, usize) {
        let len = code[at..].len() - code[at..].trim_start_matches(is_ident_char).len();
        (&code[at..at + len], at + len)
    }

    /// Byte offset of the bracket closing the one at `open`. String literals don't count.
    pub fn closing(code: &str, open: usize) -> Option<usize> {
        let mut depth = 0i32;
        let mut at = open;
        while let Some(c) = code[at..].chars().next() {
            match c {
                '"' => {
                    at += string_end(&code[at..]);
                    continue;
                }
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(at);
                    }
                }
                _ => {}
            }
            at += c.len_utf8();
        }
        None
    }

    /// The byte range of each top-level argument in `inner`.
    /// A trailing comma adds none, and commas in strings don't count.
    pub fn arg_spans(inner: &str, nesting: Nesting) -> Vec<Range<usize>> {
        let types = nesting == Nesting::Types;
        let mut spans = Vec::new();
        let mut depth = 0i32;
        let mut start = 0;
        let mut at = 0;
        while let Some(c) = inner[at..].chars().next() {
            match c {
                '"' => {
                    at += string_end(&inner[at..]);
                    continue;
                }
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => depth -= 1,
                '<' if types => depth += 1,
                '>' if types && !inner[..at].ends_with('-') => depth -= 1,
                ',' if depth == 0 => {
                    spans.push(start..at);
                    start = at + 1;
                }
                _ => {}
            }
            at += c.len_utf8();
        }
        spans.push(start..inner.len());
        if spans.last().is_some_and(|s| inner[s.clone()].trim().is_empty()) {
            spans.pop();
        }
        spans
    }

    /// The top-level arguments of a call, trimmed.
    pub fn split_args(inner: &str) -> Vec<String> {
        arg_spans(inner, Nesting::Call)
            .into_iter()
            .map(|span| inner[span].trim().to_string())
            .collect()
    }

    /// Whether `c` is an ASCII letter, digit or `_`.
    pub fn is_ident_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }
}
