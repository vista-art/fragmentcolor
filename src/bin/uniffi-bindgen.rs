// Entry point for the `uniffi-bindgen` CLI helper. Mobile build scripts run
// `cargo run --bin uniffi-bindgen generate --library ...` against the
// compiled fragmentcolor library to emit Swift/Kotlin bindings.
//
// See: https://mozilla.github.io/uniffi-rs/latest/tutorial/foreign_language_bindings.html
//
// After a Kotlin run it gives `FragmentColorException` the `message` every other
// binding has: the error's text, from the `message` field each case carries.
// uniffi's Kotlin template makes `message` list a case's fields instead, and its
// own `message` field then clashes with `Throwable.message`.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    uniffi::uniffi_bindgen_main();
    let Some(out_dir) = kotlin_out_dir(std::env::args().skip(1).collect()) else {
        return ExitCode::SUCCESS;
    };
    match fix_kotlin_messages(&out_dir.join("org/fragmentcolor/fragmentcolor.kt")) {
        Ok(()) => ExitCode::SUCCESS,
        Err(reason) => {
            eprintln!("uniffi-bindgen: {reason}");
            ExitCode::FAILURE
        }
    }
}

/// The `--out-dir` of a `generate` run for Kotlin, or `None` for any other run.
fn kotlin_out_dir(args: Vec<String>) -> Option<PathBuf> {
    let mut language = None;
    let mut out_dir = None;
    let mut generate = false;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "generate" => generate = true,
            "--language" | "-l" => language = args.next(),
            "--out-dir" | "-o" => out_dir = args.next(),
            other => {
                if let Some(value) = other.strip_prefix("--language=") {
                    language = Some(value.to_string());
                } else if let Some(value) = other.strip_prefix("--out-dir=") {
                    out_dir = Some(value.to_string());
                }
            }
        }
    }
    match (generate, language.as_deref(), out_dir) {
        (true, Some("kotlin"), Some(dir)) => Some(PathBuf::from(dir)),
        _ => None,
    }
}

/// Makes each `FragmentColorException` case's `message` field override
/// `Throwable.message`, and drops the getter that listed the fields in its place.
fn fix_kotlin_messages(path: &Path) -> Result<(), String> {
    let source = std::fs::read_to_string(path)
        .map_err(|error| format!("can't read {}: {error}", path.display()))?;
    let fixed = override_messages(&source)?;
    std::fs::write(path, fixed).map_err(|error| format!("can't write {}: {error}", path.display()))
}

/// The Kotlin source with the fix `fix_kotlin_messages` describes, or why it can't apply.
fn override_messages(source: &str) -> Result<String, String> {
    const CLASS: &str = "sealed class FragmentColorException";
    const FIELD: &str = "val `message`: kotlin.String";
    let changed = || {
        format!("uniffi's Kotlin template for {CLASS} changed; update src/bin/uniffi-bindgen.rs")
    };
    let start = source.find(CLASS).ok_or_else(changed)?;
    let handler = start
        + source[start..]
            .find("companion object ErrorHandler")
            .ok_or_else(changed)?;
    // The class's lines, up to the line that opens its error handler.
    let end = source[..handler]
        .rfind('\n')
        .map_or(start, |newline| newline + 1);
    let mut lines = source[start..end].lines().peekable();
    let mut class = String::new();
    let (mut fields, mut getters) = (0, 0);
    while let Some(line) = lines.next() {
        let next = lines.peek().map(|next| next.trim_start());
        if line.trim() == "override val message"
            && next.is_some_and(|next| next.starts_with("get() = \"message="))
        {
            lines.next();
            getters += 1;
            continue;
        }
        if line.trim_start().starts_with(FIELD) {
            class.push_str(&line.replacen(FIELD, &format!("override {FIELD}"), 1));
            fields += 1;
        } else {
            class.push_str(line);
        }
        class.push('\n');
    }
    if fields == 0 || fields != getters {
        return Err(changed());
    }
    Ok(format!("{}{class}{}", &source[..start], &source[end..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Story: a Kotlin run gets its out dir back, whichever way the flags are spelled,
    // and a Swift run gets none, so only Kotlin output is changed.
    #[test]
    fn reads_kotlin_runs_only() {
        let args = |line: &str| line.split(' ').map(String::from).collect::<Vec<_>>();
        assert_eq!(
            kotlin_out_dir(args(
                "generate --library x.so --language kotlin --out-dir out"
            )),
            Some(PathBuf::from("out"))
        );
        assert_eq!(
            kotlin_out_dir(args("generate -l kotlin -o out --library x.so")),
            Some(PathBuf::from("out"))
        );
        assert_eq!(
            kotlin_out_dir(args("generate --language=kotlin --out-dir=out")),
            Some(PathBuf::from("out"))
        );
        assert_eq!(
            kotlin_out_dir(args("generate --language swift --out-dir out")),
            None
        );
    }

    // Story: each case's `message` field overrides `Throwable.message`, so Kotlin's
    // `message` is the error's text, and the getter that listed the fields is gone.
    #[test]
    fn message_holds_the_text() {
        let source = r#"before
sealed class FragmentColorException: kotlin.Exception() {
    class Other(

        val `message`: kotlin.String
        ) : FragmentColorException() {
        override val message
            get() = "message=${ `message` }"
    }
    class Entry(

        val `message`: kotlin.String,

        val `stage`: kotlin.String
        ) : FragmentColorException() {
        override val message
            get() = "message=${ `message` }, stage=${ `stage` }"
    }
    companion object ErrorHandler : UniffiRustCallStatusErrorHandler<FragmentColorException> {
    }
}
"#;

        let fixed = override_messages(source).ok();

        let expected = r#"before
sealed class FragmentColorException: kotlin.Exception() {
    class Other(

        override val `message`: kotlin.String
        ) : FragmentColorException() {
    }
    class Entry(

        override val `message`: kotlin.String,

        val `stage`: kotlin.String
        ) : FragmentColorException() {
    }
    companion object ErrorHandler : UniffiRustCallStatusErrorHandler<FragmentColorException> {
    }
}
"#;
        assert_eq!(fixed.as_deref(), Some(expected));
    }

    // Story: a template that no longer has the shape this fix expects fails the run
    // instead of leaving Kotlin's `message` listing fields.
    #[test]
    fn refuses_a_changed_template() {
        assert!(override_messages("class Other").is_err());
        let source = "sealed class FragmentColorException {\n    class Other(\n        val `message`: kotlin.String\n    )\n    companion object ErrorHandler {}\n";
        assert!(override_messages(source).is_err());
    }
}
