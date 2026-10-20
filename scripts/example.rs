mod example {
    //! A doc example split into the lines every language shares
    //! and the hidden lines one language needs around them.
    //!
    //! Rust hides a doctest line with `# `. Another language's hidden lines sit in a
    //! hidden Rust block comment tagged with the language, so rustdoc compiles them as a comment:
    //!
    //! ```text
    //! # /*[js]
    //! # const canvas = document.createElement("canvas");
    //! # */
    //! ```
    //!
    //! A block before the first visible line goes after the imports of the transpiled lines,
    //! and a block after the last visible line goes after them.

    use std::collections::BTreeMap;

    /// A language an example transpiles to.
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub enum Lang {
        Js,
        Python,
        Swift,
        Kotlin,
    }

    impl Lang {
        const ALL: [Lang; 4] = [Lang::Js, Lang::Python, Lang::Swift, Lang::Kotlin];

        /// The tag that opens this language's hidden block.
        fn tag(self) -> &'static str {
            match self {
                Lang::Js => "js",
                Lang::Python => "python",
                Lang::Swift => "swift",
                Lang::Kotlin => "kotlin",
            }
        }
    }

    /// The hidden lines one language puts around the shared lines.
    #[derive(Default)]
    pub struct Harness {
        before: Vec<String>,
        after: Vec<String>,
    }

    impl Harness {
        /// `body` with the hidden lines around it, and the 1-based line ranges they take.
        /// The lines before go after the imports `body` opens with.
        pub fn wrap(&self, body: &str) -> (String, Vec<(usize, usize)>) {
            if self.before.is_empty() && self.after.is_empty() {
                return (body.to_string(), Vec::new());
            }
            let mut lines: Vec<&str> = body.lines().collect();
            let mut ranges = Vec::new();
            if !self.before.is_empty() {
                let at = imports_end(&lines);
                lines.splice(at..at, self.before.iter().map(String::as_str));
                ranges.push((at + 1, at + self.before.len()));
            }
            if !self.after.is_empty() {
                let first = lines.len() + 1;
                lines.extend(self.after.iter().map(String::as_str));
                ranges.push((first, lines.len()));
            }
            let mut text = lines.join("\n");
            if body.ends_with('\n') && self.after.is_empty() {
                text.push('\n');
            }
            (text, ranges)
        }
    }

    /// The index just past the import lines that open `lines`, or 0.
    fn imports_end(lines: &[&str]) -> usize {
        let mut end = 0;
        for (at, line) in lines.iter().enumerate() {
            let line = line.trim();
            if line.starts_with("import ") || (line.starts_with("from ") && line.contains(" import ")) {
                end = at + 1;
            } else if !line.is_empty() {
                break;
            }
        }
        end
    }

    /// A Rust doc example and the hidden lines of every other language.
    pub struct Example {
        /// The Rust lines, each `true` when hidden, without any other language's lines.
        pub rust: Vec<(String, bool)>,
        harness: BTreeMap<Lang, Harness>,
    }

    impl Example {
        /// Split a Rust code fence. Fails on a block it can't place.
        pub fn parse(code: &str) -> Result<Example, String> {
            let mut rust: Vec<(String, bool)> = Vec::new();
            // Each block, with the number of Rust lines before it.
            let mut blocks: Vec<(Lang, usize, Vec<String>)> = Vec::new();
            let mut open: Option<(Lang, Vec<String>)> = None;
            for line in code.lines() {
                let hidden = line.trim_start().starts_with('#');
                let text = if hidden { hidden_text(line) } else { line.to_string() };
                if let Some((lang, lines)) = open.as_mut() {
                    if !hidden {
                        return Err(format!("a `{line}` line is visible inside the [{}] block", lang.tag()));
                    }
                    if text.trim() == "*/" {
                        blocks.push((*lang, rust.len(), std::mem::take(lines)));
                        open = None;
                    } else {
                        lines.push(text);
                    }
                    continue;
                }
                match hidden.then(|| opener(&text)).flatten() {
                    Some(lang) => open = Some((lang?, Vec::new())),
                    None => rust.push((text, hidden)),
                }
            }
            if let Some((lang, _)) = open {
                return Err(format!("the [{}] block has no closing `# */`", lang.tag()));
            }

            let shows = |(text, hidden): &(String, bool)| !hidden && !text.trim().is_empty();
            let first = rust.iter().position(shows).unwrap_or(rust.len());
            let end = rust.iter().rposition(shows).map_or(0, |at| at + 1);
            let mut harness: BTreeMap<Lang, Harness> = BTreeMap::new();
            for (lang, at, lines) in blocks {
                let side = harness.entry(lang).or_default();
                if at <= first {
                    side.before.extend(lines);
                } else if at >= end {
                    side.after.extend(lines);
                } else {
                    return Err(format!(
                        "the [{}] block sits between visible lines; move it before or after them",
                        lang.tag()
                    ));
                }
            }
            Ok(Example { rust, harness })
        }

        /// The hidden lines `lang` puts around the shared lines. Empty when it has none.
        pub fn harness(&self, lang: Lang) -> &Harness {
            static EMPTY: Harness = Harness {
                before: Vec::new(),
                after: Vec::new(),
            };
            self.harness.get(&lang).unwrap_or(&EMPTY)
        }
    }

    /// A hidden line without its `#` and the one space after it.
    fn hidden_text(line: &str) -> String {
        let trimmed = line.trim_start();
        let indent = &line[..line.len() - trimmed.len()];
        let after = trimmed.strip_prefix('#').unwrap_or(trimmed);
        format!("{indent}{}", after.strip_prefix(' ').unwrap_or(after))
    }

    /// The language a `/*[tag]` line opens a block for, `None` for any other line.
    fn opener(text: &str) -> Option<Result<Lang, String>> {
        let rest = text.trim().strip_prefix("/*[")?;
        let (tag, tail) = rest.split_once(']')?;
        let Some(lang) = Lang::ALL.into_iter().find(|lang| lang.tag() == tag) else {
            let tags: Vec<&str> = Lang::ALL.iter().map(|lang| lang.tag()).collect();
            return Some(Err(format!("`[{tag}]` names no language; use one of {tags:?}")));
        };
        if !tail.trim().is_empty() {
            return Some(Err(format!("put the [{tag}] block's lines below `# /*[{tag}]`, one per line")));
        }
        Some(Ok(lang))
    }
}
