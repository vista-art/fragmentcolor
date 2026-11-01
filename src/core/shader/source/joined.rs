//! WGSL parts joined into one source, each later part's directives
//! moved to the top, the only place WGSL takes them. naga's reports on it
//! point at the line and column where the parts, joined as written, hold each place.

use crate::ShaderError;
use crate::core::shader::error::{Diagnostic, Source};
use crate::core::shader::reflection::{create_parser, create_validator};
use crate::core::shader::{parse_wgsl, validate_module};
use naga::front::wgsl::ParseError;
use naga::valid::ModuleInfo;
use naga::{Module, Span, WithSpan};
use std::error::Error;
use std::fmt;
use std::iter::repeat_n;
use std::ops::Range;

/// What goes between two parts.
const SEPARATOR: &str = "\n\n";

/// The words that open a WGSL global directive.
const DIRECTIVES: [&str; 3] = ["enable", "requires", "diagnostic"];

/// WGSL joined from parts in order, the text naga parses.
pub(crate) struct Joined {
    /// The text naga parses.
    pub(crate) text: String,
    /// `None` when `text` is the parts as written,
    /// else where the directives at the top of `text` came from.
    moves: Option<Moves>,
}

/// The parts as written, and where each moved directive sits in them.
struct Moves {
    /// The parts joined as written, where naga's reports point.
    written: String,
    /// The bytes the moved directives take at the top of the joined text.
    top: usize,
    /// Each moved directive: its range at the top of the joined text, and its start in `written`.
    places: Vec<(Range<usize>, usize)>,
}

impl From<&str> for Joined {
    fn from(source: &str) -> Self {
        Self {
            text: source.to_string(),
            moves: None,
        }
    }
}

impl Joined {
    /// `parts` joined in order, with the directives each part after the first
    /// opens with moved to the top. Without such directives the text is the parts as written.
    pub(crate) fn join(parts: &[&str]) -> Self {
        let written = parts.join(SEPARATOR);
        let mut moved = Vec::new();
        let mut start = 0;
        for (index, part) in parts.iter().enumerate() {
            if index > 0 {
                let found = find_directives(part).into_iter();
                moved.extend(found.map(|range| start + range.start..start + range.end));
            }
            start += part.len() + SEPARATOR.len();
        }
        if moved.is_empty() {
            return Self {
                text: written,
                moves: None,
            };
        }
        let mut text = String::with_capacity(written.len() * 2);
        let mut places = Vec::with_capacity(moved.len());
        let directives = moved
            .iter()
            .filter_map(|range| Some((range, written.get(range.clone())?)));
        for (range, directive) in directives {
            let at = text.len();
            text.push_str(directive);
            places.push((at..text.len(), range.start));
            text.push('\n');
        }
        let top = text.len();
        text.push_str(&blank(&written, &moved));
        let moves = Moves {
            written,
            top,
            places,
        };
        Self {
            text,
            moves: Some(moves),
        }
    }

    /// Parses the joined text, or returns naga's report, which points into the parts as written.
    pub(crate) fn parse(&self) -> Result<Module, ShaderError> {
        let Some(moves) = &self.moves else {
            return parse_wgsl(&self.text);
        };
        match create_parser().parse(&self.text) {
            Ok(module) => Ok(module),
            Err(error) => Err(ShaderError::WgslParseError(moves.report_parse(&error))),
        }
    }

    /// Validates `module`, parsed from the joined text,
    /// or returns naga's report, which points into the parts as written.
    pub(crate) fn validate(&self, module: &Module) -> Result<ModuleInfo, ShaderError> {
        if self.moves.is_none() {
            return validate_module(module, Some(Source::wgsl(&self.text)));
        }
        match create_validator().validate(module) {
            Ok(info) => Ok(info),
            Err(error) => Err(ShaderError::ValidationError(self.report(&error))),
        }
    }

    /// The report on `error`, whose places are in the joined text,
    /// with each place moved to where the parts as written hold it.
    pub(crate) fn report<E: Error + Clone>(&self, error: &WithSpan<E>) -> Diagnostic {
        match &self.moves {
            Some(moves) => moves.report_validation(error),
            None => Diagnostic::from_spans(error, Some(Source::wgsl(&self.text))),
        }
    }
}

impl Moves {
    /// naga's report on a parse error, at the places in `written`.
    fn report_parse(&self, error: &ParseError) -> Diagnostic {
        let labels = error
            .labels()
            .map(|(span, label)| (span, label.to_string()));
        let moved = self.relocate(Report::from(error), labels);
        Diagnostic::from_spans(&moved, Some(Source::wgsl(&self.written)))
    }

    /// The report on an error that holds its places, at the places in `written`.
    fn report_validation<E: Error + Clone>(&self, error: &WithSpan<E>) -> Diagnostic {
        let moved = self.relocate(error.as_inner().clone(), error.spans().cloned());
        Diagnostic::from_spans(&moved, Some(Source::wgsl(&self.written)))
    }

    /// `inner` with each of `spans`, a place in the joined text, moved to its place in `written`.
    fn relocate<E>(&self, inner: E, spans: impl Iterator<Item = (Span, String)>) -> WithSpan<E> {
        spans.fold(WithSpan::new(inner), |error, (span, label)| {
            error.with_span(self.place(span), label)
        })
    }

    /// The place in `written` of `span`, a place in the joined text.
    /// One that lands nowhere in `written` comes back undefined, which reports skip.
    fn place(&self, span: Span) -> Span {
        let Some(range) = span.to_range() else {
            return span;
        };
        let Some(start) = self.locate(range.start) else {
            return Span::UNDEFINED;
        };
        let end = start + range.len();
        let bounds = (u32::try_from(start), u32::try_from(end));
        match (self.written.get(start..end), bounds) {
            (Some(_), (Ok(start), Ok(end))) => Span::new(start, end),
            _ => Span::UNDEFINED,
        }
    }

    /// The byte of `written` that the byte `at` of the joined text came from.
    fn locate(&self, at: usize) -> Option<usize> {
        if at >= self.top {
            return Some(at - self.top);
        }
        self.places
            .iter()
            .find(|(range, _)| range.contains(&at))
            .map(|(range, start)| start + (at - range.start))
    }
}

/// A parse error's message, then each of its notes, as a chain of errors.
/// naga writes the chain under a spanned error as notes, so its report reads as a parse error's.
#[derive(Debug)]
struct Report {
    text: String,
    next: Option<Box<Report>>,
}

impl From<&ParseError> for Report {
    fn from(error: &ParseError) -> Self {
        let notes: Vec<&str> = error.notes().collect();
        let next = notes.into_iter().rev().fold(None, |next, note| {
            let text = format!("note: {note}");
            Some(Box::new(Report { text, next }))
        });
        Self {
            text: error.message().to_string(),
            next,
        }
    }
}

impl fmt::Display for Report {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.text)
    }
}

impl Error for Report {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.next
            .as_deref()
            .map(|next| next as &(dyn Error + 'static))
    }
}

/// The directives `part` opens with, each a byte range
/// that ends past its `;`, all before its first declaration.
fn find_directives(part: &str) -> Vec<Range<usize>> {
    let mut found = Vec::new();
    let mut at = skip_trivia(part, 0);
    while let Some(end) = read_directive(part, at) {
        found.push(at..end);
        at = skip_trivia(part, end);
    }
    found
}

/// The end of the directive that starts at `at`, past its `;`, when one starts there.
fn read_directive(text: &str, at: usize) -> Option<usize> {
    let rest = text.get(at..)?;
    let word = rest
        .split(|character: char| !(character.is_alphanumeric() || character == '_'))
        .next()?;
    if !DIRECTIVES.contains(&word) {
        return None;
    }
    let mut end = at + word.len();
    loop {
        end = skip_trivia(text, end);
        let character = text.get(end..)?.chars().next()?;
        end += character.len_utf8();
        if character == ';' {
            return Some(end);
        }
    }
}

/// The first byte at or after `at` that starts neither blankspace nor a comment.
fn skip_trivia(text: &str, mut at: usize) -> usize {
    while let Some(rest) = text.get(at..) {
        let first = rest.chars().next();
        if let Some(character) = first.filter(|character| is_blank(*character)) {
            at += character.len_utf8();
        } else if rest.starts_with("//") {
            at += rest.find(is_line_break).unwrap_or(rest.len());
        } else if rest.starts_with("/*") {
            at += measure_block(rest);
        } else {
            return at;
        }
    }
    text.len()
}

/// The bytes of the block comment `text` opens with,
/// nested comments included, or all of `text` when it never closes.
fn measure_block(text: &str) -> usize {
    let mut depth = 0;
    let mut at = 0;
    while let Some(rest) = text.get(at..) {
        if rest.starts_with("/*") {
            depth += 1;
            at += 2;
        } else if rest.starts_with("*/") {
            depth -= 1;
            at += 2;
            if depth == 0 {
                return at;
            }
        } else if let Some(character) = rest.chars().next() {
            at += character.len_utf8();
        } else {
            break;
        }
    }
    text.len()
}

/// Whether WGSL reads `character` as blankspace.
fn is_blank(character: char) -> bool {
    matches!(character, ' ' | '\t' | '\u{200E}' | '\u{200F}') || is_line_break(character)
}

/// Whether WGSL reads `character` as a line break, which ends a line comment.
fn is_line_break(character: char) -> bool {
    matches!(
        character,
        '\n' | '\u{0B}' | '\u{0C}' | '\r' | '\u{85}' | '\u{2028}' | '\u{2029}'
    )
}

/// `text` with each character in `ranges` but line breaks
/// turned to spaces of the same byte length, so no other byte moves.
fn blank(text: &str, ranges: &[Range<usize>]) -> String {
    let mut blanked = String::with_capacity(text.len());
    for (at, character) in text.char_indices() {
        let moved = ranges.iter().any(|range| range.contains(&at));
        if moved && !is_line_break(character) {
            blanked.extend(repeat_n(' ', character.len_utf8()));
        } else {
            blanked.push(character);
        }
    }
    blanked
}

#[cfg(test)]
mod tests {
    use super::*;

    // Story: a part that opens with directives gives each one as a range that ends at its `;`,
    // past comments and blank lines, and stops at the first declaration.
    #[test]
    fn finds_the_directives_a_part_opens_with() {
        let part = "\n  // helpers\n  enable primitive_index; /* a ; in a comment */\n  diagnostic(off, derivative_uniformity) /* ; */ ;\nrequires readonly_and_readwrite_storage_textures;\nfn helper() {}\nenable f16;";
        let found: Vec<&str> = find_directives(part)
            .into_iter()
            .map(|range| &part[range])
            .collect();
        assert_eq!(
            found,
            [
                "enable primitive_index;",
                "diagnostic(off, derivative_uniformity) /* ; */ ;",
                "requires readonly_and_readwrite_storage_textures;",
            ]
        );
        assert!(find_directives("fn enable() {}").is_empty());
        assert!(find_directives("enabled;").is_empty());
        assert!(find_directives("/* never closes enable f16;").is_empty());
    }

    // Story: parts with no directive past the first join as written, and moved directives
    // leave every other byte of the parts where the written text holds it.
    #[test]
    fn moves_only_the_directives_of_later_parts() {
        let first = "enable primitive_index;\nfn a() {}";
        let plain = Joined::join(&[first, "fn b() {}"]);
        assert_eq!(plain.text, format!("{first}\n\nfn b() {{}}"));
        assert!(plain.moves.is_none());

        let joined = Joined::join(&[first, "diagnostic(off, derivative_uniformity);\nfn b() {}"]);
        let Some(moves) = &joined.moves else {
            panic!("expected a moved directive");
        };
        assert!(
            joined
                .text
                .starts_with("diagnostic(off, derivative_uniformity);\nenable primitive_index;"),
            "{}",
            joined.text
        );
        let body = &joined.text[moves.top..];
        assert_eq!(body.len(), moves.written.len());
        assert!(!body.contains("diagnostic"), "{body}");
        let declaration = moves.written.find("fn b").expect("a declaration");
        assert_eq!(moves.locate(moves.top + declaration), Some(declaration));
        let directive = moves.written.find("diagnostic").expect("the directive");
        assert_eq!(moves.locate(2), Some(directive + 2));
    }

    // Story: a report naga writes through `relocate` reads as naga's own report on a parse
    // error, notes included, so a moved place changes the numbers in a message and nothing else.
    #[test]
    fn relocated_reports_read_as_naga_reports() {
        let nested = format!("fn a() {{ {}{} }}", "{".repeat(130), "}".repeat(130));
        let sources = ["fn a() -> f32 {\n  return 1.0 +;\n}", nested.as_str()];
        for (index, source) in sources.into_iter().enumerate() {
            let error = create_parser().parse(source).expect_err("a parse error");
            assert_eq!(error.notes().len() > 0, index == 1, "{error}");
            let moves = Moves {
                written: source.to_string(),
                top: 0,
                places: Vec::new(),
            };

            let relocated = moves.report_parse(&error);

            assert_eq!(relocated, Diagnostic::from_parse(&error, source));
        }
    }
}
