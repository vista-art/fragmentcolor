//! Doc link utilities for testing link rewriting behavior.
//! These functions intentionally duplicate the logic used in the build script
//! so we can unit-test expected behavior without invoking the build.

use std::collections::HashSet;

/// Normalize links in MDX content to a root (e.g., "/api" or a configured base).
/// - Absolute links to https://fragmentcolor.org/api/... (http/https, with/without www) get
///   rewritten to {base}/... preserving the path.
/// - Category-relative links like "core/renderer" get rewritten to {base}/core/renderer
///   if the first segment is in `top_categories`.
pub fn rewrite_links_to_site_root(
    mdx: &str,
    top_categories: &HashSet<String>,
    base: &str,
) -> String {
    let mut out = String::new();
    let bytes = mdx.as_bytes();
    let mut i = 0usize;

    let n_https = "](https://fragmentcolor.org/api/".as_bytes();
    let n_http = "](http://fragmentcolor.org/api/".as_bytes();
    let n_www_https = "](https://www.fragmentcolor.org/api/".as_bytes();
    let n_www_http = "](http://www.fragmentcolor.org/api/".as_bytes();

    while i < bytes.len() {
        let mut matched = None::<&[u8]>;
        if i + n_https.len() <= bytes.len() && &bytes[i..i + n_https.len()] == n_https {
            matched = Some(n_https);
        } else if i + n_http.len() <= bytes.len() && &bytes[i..i + n_http.len()] == n_http {
            matched = Some(n_http);
        } else if i + n_www_https.len() <= bytes.len()
            && &bytes[i..i + n_www_https.len()] == n_www_https
        {
            matched = Some(n_www_https);
        } else if i + n_www_http.len() <= bytes.len()
            && &bytes[i..i + n_www_http.len()] == n_www_http
        {
            matched = Some(n_www_http);
        }

        if let Some(m) = matched {
            out.push_str("](");
            i += m.len();
            if !base.is_empty() {
                out.push_str(base);
                if !base.ends_with('/') {
                    out.push('/');
                } else if base == "/" {
                    // keep single slash only
                }
            }
            while i < bytes.len() && bytes[i] != b')' {
                out.push(bytes[i] as char);
                i += 1;
            }
        } else if i + 2 <= bytes.len() && bytes[i] == b']' && bytes[i + 1] == b'(' {
            // Generic link: ](href)
            out.push(']');
            out.push('(');
            i += 2;
            let start = i;
            while i < bytes.len() && bytes[i] != b')' {
                i += 1;
            }
            let href = &mdx[start..i];
            let href_trim = href.trim();
            let lower = href_trim.to_ascii_lowercase();
            let is_abs = lower.starts_with("http://")
                || lower.starts_with("https://")
                || href_trim.starts_with('/')
                || href_trim.starts_with('#')
                || lower.starts_with("mailto:")
                || lower.starts_with("tel:")
                || lower.starts_with("data:");
            if is_abs {
                out.push_str(href_trim);
            } else {
                let top = href_trim.split('/').next().unwrap_or("");
                if top_categories.contains(top) {
                    if !base.is_empty() {
                        out.push_str(base);
                        if !href_trim.starts_with('/') {
                            out.push('/');
                        }
                    }
                    out.push_str(href_trim);
                } else {
                    out.push_str(href_trim);
                }
            }
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out
}
