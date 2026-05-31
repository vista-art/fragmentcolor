//! locks — inspect the lock-block versioning store.
//!
//! Reads `docs/website/.locks/locks.json` (written by the website's
//! Astro integration at `docs/website/integrations/locks.ts`) and
//! surfaces it for review:
//!
//!   cargo run --release -p fce --example locks -- status
//!     One line per (post, lock_id), with current version + last
//!     updated time + drift indicator. Quick scan after a sub-agent
//!     run: anything bumped to a new version is something they
//!     touched inside a lock.
//!
//!   cargo run --release -p fce --example locks -- history <post> [<lock_id>]
//!     Full version trail for one or every lock in a post. <post> is
//!     the path under the repo root, or just the bare filename
//!     (matched by suffix). Shows version, hash, save time, and the
//!     description/comments that were live at that version.
//!
//!   cargo run --release -p fce --example locks -- diff <post> <lock_id> [<vA>] [<vB>]
//!     Unified diff between two versions. With no version args, diffs
//!     the most recent two. With one, diffs that one against the
//!     current version.
//!
//! Convention: this binary is read-only. The website's Astro
//! integration owns writes — manual tweaks to the JSON are not
//! supported and will likely round-trip away on the next dev-server
//! save or production build.

use serde::Deserialize;
use similar::{ChangeTag, TextDiff};
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

const STORE_REL: &str = "docs/website/.locks/locks.json";

#[derive(Deserialize, Default)]
struct Store {
    blocks: Vec<Block>,
}

// Mirror of scripts/locks.rs. `current_content` is unused here (we
// diff via history entries instead) but kept so the schema round-trips
// cleanly if anything else ever serializes the store back.
#[allow(dead_code)]
#[derive(Deserialize, Clone)]
struct Block {
    post_id: String,
    lock_id: String,
    current_version: u64,
    current_hash: String,
    current_content: String,
    description: Option<String>,
    comments: Option<String>,
    updated_at: u64,
    history: Vec<Snapshot>,
}

#[allow(dead_code)]
#[derive(Deserialize, Clone)]
struct Snapshot {
    version: u64,
    hash: String,
    content: String,
    saved_at: u64,
    description: Option<String>,
    comments: Option<String>,
}

fn workspace_root() -> PathBuf {
    let cargo_manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    if cargo_manifest.is_empty() {
        return std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    }
    let p = PathBuf::from(cargo_manifest);
    // examples/rust -> repo root is two levels up.
    p.parent()
        .and_then(|p| p.parent())
        .map_or(p.clone(), |p| p.to_path_buf())
}

fn load_store() -> Result<Store, String> {
    let path = workspace_root().join(STORE_REL);
    let s = fs::read_to_string(&path).map_err(|e| format!("read {}: {}", path.display(), e))?;
    serde_json::from_str(&s).map_err(|e| format!("parse {}: {}", path.display(), e))
}

fn fmt_age(saved_at: u64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if saved_at == 0 || saved_at > now {
        return "—".to_string();
    }
    let delta = now - saved_at;
    if delta < 60 {
        format!("{}s ago", delta)
    } else if delta < 3600 {
        format!("{}m ago", delta / 60)
    } else if delta < 86_400 {
        format!("{}h ago", delta / 3600)
    } else {
        format!("{}d ago", delta / 86_400)
    }
}

/// Match `post` against the stored post_ids. Accepts either a full path
/// (matched as substring) or a basename. Returns the matching post_id,
/// or an error listing candidates if zero/multiple match.
fn resolve_post(store: &Store, query: &str) -> Result<String, String> {
    let mut posts: Vec<String> = store.blocks.iter().map(|b| b.post_id.clone()).collect();
    posts.sort();
    posts.dedup();

    let exact: Vec<&String> = posts.iter().filter(|p| p.as_str() == query).collect();
    if exact.len() == 1 {
        return Ok(exact[0].clone());
    }

    let suffix: Vec<&String> = posts
        .iter()
        .filter(|p| p.ends_with(query) || p.contains(query))
        .collect();
    match suffix.len() {
        0 => Err(format!(
            "no post matches `{}`. Known posts:\n  {}",
            query,
            posts.join("\n  ")
        )),
        1 => Ok(suffix[0].clone()),
        _ => Err(format!(
            "`{}` matches multiple posts:\n  {}",
            query,
            suffix
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join("\n  ")
        )),
    }
}

fn cmd_status(store: &Store) {
    if store.blocks.is_empty() {
        println!(
            "No lock blocks tracked yet. Add `<Lock id=\"...\">...</Lock>` to an MDX file and rebuild."
        );
        return;
    }
    // Group by post_id.
    let mut by_post: std::collections::BTreeMap<String, Vec<&Block>> =
        std::collections::BTreeMap::new();
    for b in &store.blocks {
        by_post.entry(b.post_id.clone()).or_default().push(b);
    }
    for (post, blocks) in by_post {
        println!("\n{}", post);
        let mut blocks = blocks.clone();
        blocks.sort_by(|a, b| a.lock_id.cmp(&b.lock_id));
        for b in blocks {
            let desc = b
                .description
                .as_deref()
                .map(|d| format!(" — {}", d))
                .unwrap_or_default();
            println!(
                "  {:<24} v{:<3} {:<16} {:<12}{}",
                b.lock_id,
                b.current_version,
                short_hash(&b.current_hash),
                fmt_age(b.updated_at),
                desc
            );
        }
    }
}

fn short_hash(h: &str) -> String {
    if h.len() > 8 {
        h[..8].to_string()
    } else {
        h.to_string()
    }
}

fn cmd_history(store: &Store, post_query: &str, lock_filter: Option<&str>) -> Result<(), String> {
    let post = resolve_post(store, post_query)?;
    let blocks: Vec<&Block> = store
        .blocks
        .iter()
        .filter(|b| b.post_id == post && lock_filter.map(|f| b.lock_id == f).unwrap_or(true))
        .collect();

    if blocks.is_empty() {
        return Err(format!(
            "no locks in `{}`{}",
            post,
            lock_filter
                .map(|f| format!(" matching id `{}`", f))
                .unwrap_or_default()
        ));
    }

    println!("{}\n", post);
    for b in blocks {
        println!(
            "[{}] (currently v{}, hash {})",
            b.lock_id,
            b.current_version,
            short_hash(&b.current_hash)
        );
        if let Some(d) = &b.description {
            println!("  description: {}", d);
        }
        if let Some(c) = &b.comments {
            println!("  comments:    {}", c);
        }
        for s in &b.history {
            let tag = if s.version == b.current_version {
                "*"
            } else {
                " "
            };
            let dup = if s.version != b.current_version && s.hash == b.current_hash {
                " (matches current)"
            } else {
                ""
            };
            println!(
                "  {} v{:<3} {:<16} {:<14}{}",
                tag,
                s.version,
                short_hash(&s.hash),
                fmt_age(s.saved_at),
                dup
            );
        }
        println!();
    }
    Ok(())
}

fn pick_versions(
    b: &Block,
    va: Option<u64>,
    vb: Option<u64>,
) -> Result<(Snapshot, Snapshot), String> {
    let find = |v: u64| -> Option<Snapshot> { b.history.iter().find(|s| s.version == v).cloned() };
    match (va, vb) {
        (None, None) => {
            let n = b.history.len();
            if n < 2 {
                return Err("need at least two versions to diff".into());
            }
            Ok((b.history[n - 2].clone(), b.history[n - 1].clone()))
        }
        (Some(a), None) => {
            let sa = find(a).ok_or_else(|| format!("v{} not found", a))?;
            let cur = b
                .history
                .last()
                .cloned()
                .ok_or_else(|| "history empty".to_string())?;
            Ok((sa, cur))
        }
        (Some(a), Some(c)) => {
            let sa = find(a).ok_or_else(|| format!("v{} not found", a))?;
            let sc = find(c).ok_or_else(|| format!("v{} not found", c))?;
            Ok((sa, sc))
        }
        (None, Some(_)) => unreachable!("CLI parser collapses this case"),
    }
}

fn cmd_diff(
    store: &Store,
    post_query: &str,
    lock_id: &str,
    va: Option<u64>,
    vb: Option<u64>,
) -> Result<(), String> {
    let post = resolve_post(store, post_query)?;
    let block = store
        .blocks
        .iter()
        .find(|b| b.post_id == post && b.lock_id == lock_id)
        .ok_or_else(|| format!("lock `{}` not found in `{}`", lock_id, post))?;

    let (a, c) = pick_versions(block, va, vb)?;
    println!(
        "{} :: {}\nv{} ({}) -> v{} ({})\n",
        post,
        lock_id,
        a.version,
        short_hash(&a.hash),
        c.version,
        short_hash(&c.hash)
    );
    let diff = TextDiff::from_lines(&a.content, &c.content);
    for change in diff.iter_all_changes() {
        let (sign, prefix) = match change.tag() {
            ChangeTag::Delete => ("-", "\x1b[31m"),
            ChangeTag::Insert => ("+", "\x1b[32m"),
            ChangeTag::Equal => (" ", ""),
        };
        let reset = if prefix.is_empty() { "" } else { "\x1b[0m" };
        print!("{}{}{}{}", prefix, sign, change.value(), reset);
    }
    Ok(())
}

fn print_help() {
    println!(
        "{}",
        "\
locks — inspect the lock-block versioning store

USAGE
    locks status
    locks history <post> [<lock_id>]
    locks diff    <post> <lock_id> [<vA>] [<vB>]

<post> can be a full repo-relative path or any unique substring/suffix.
Diff with no version args shows the most recent two; one arg diffs
that one against the current version.
"
    );
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let store = match load_store() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("locks: {}", e);
            return ExitCode::FAILURE;
        }
    };

    let result: Result<(), String> = match args.first().map(|s| s.as_str()) {
        None | Some("help") | Some("-h") | Some("--help") => {
            print_help();
            Ok(())
        }
        Some("status") => {
            cmd_status(&store);
            Ok(())
        }
        Some("history") => {
            let post = args
                .get(1)
                .ok_or_else(|| "history requires <post>".to_string());
            match post {
                Ok(post) => cmd_history(&store, post, args.get(2).map(|s| s.as_str())),
                Err(e) => Err(e),
            }
        }
        Some("diff") => {
            let post = args
                .get(1)
                .ok_or_else(|| "diff requires <post>".to_string());
            let lock_id = args
                .get(2)
                .ok_or_else(|| "diff requires <lock_id>".to_string());
            match (post, lock_id) {
                (Ok(post), Ok(lock_id)) => {
                    let va = args.get(3).and_then(|s| s.parse::<u64>().ok());
                    let vb = args.get(4).and_then(|s| s.parse::<u64>().ok());
                    cmd_diff(&store, post, lock_id, va, vb)
                }
                (Err(e), _) | (_, Err(e)) => Err(e),
            }
        }
        Some(other) => Err(format!("unknown subcommand `{}`. Try `locks help`.", other)),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("locks: {}", e);
            ExitCode::FAILURE
        }
    }
}
