mod links {
    //! Checks every fragmentcolor.org/api link in docs/api against the site's pages.
    //!
    //! The site writes one page per object, at `/api/<category>/<object>`.
    //! A method is a heading on its object's page, with the id github-slugger
    //! makes from its text, so a link names it by anchor: `/api/core/renderer#rendererload`.
    //! The same links reach the Rust hover docs and the Python stub, so they must work as written.

    use std::collections::{HashMap, HashSet};
    use std::path::{Path, PathBuf};

    const SITE: &str = "https://fragmentcolor.org/api/";
    const HOSTS: [&str; 4] = [
        "https://fragmentcolor.org/api/",
        "http://fragmentcolor.org/api/",
        "https://www.fragmentcolor.org/api/",
        "http://www.fragmentcolor.org/api/",
    ];

    /// The heading ids of one page, and the id each method's heading gets.
    #[derive(Default)]
    struct Page {
        ids: HashSet<String>,
        methods: HashMap<String, String>,
    }

    /// One problem for each link that names no page, or no heading on its page.
    pub fn check(docs_root: &Path) -> Vec<String> {
        let (pages, dirs) = read_pages(docs_root);
        let mut problems = Vec::new();
        for file in list_markdown(docs_root) {
            let Ok(text) = std::fs::read_to_string(&file) else {
                continue;
            };
            for link in find_links(&text) {
                if let Some(problem) = check_link(&pages, &dirs, link) {
                    problems.push(format!("{}: {}", file.display(), problem));
                }
            }
        }
        problems
    }

    /// The pages the site builds, keyed by URL path, and the page each docs folder becomes.
    fn read_pages(docs_root: &Path) -> (HashMap<String, Page>, HashMap<String, String>) {
        let mut pages = HashMap::new();
        let mut dirs = HashMap::new();
        for (object, dir, category) in super::docs::scan_docs_objects(docs_root) {
            if category.split('/').any(|segment| segment == "hidden") {
                continue;
            }
            let key = if category.is_empty() {
                object.to_lowercase()
            } else {
                format!("{}/{}", category, object.to_lowercase())
            };
            if let Ok(rel) = dir.strip_prefix(docs_root) {
                dirs.insert(rel.to_string_lossy().replace('\\', "/"), key.clone());
            }
            pages.insert(key, read_page(&dir));
        }
        (pages, dirs)
    }

    /// The ids of the headings the site writes for the object in `dir`.
    fn read_page(dir: &Path) -> Page {
        let mut page = Page::default();
        page.ids
            .extend(["description", "example", "methods"].map(String::from));
        let name = dir.file_name().and_then(|s| s.to_str()).unwrap_or_default();
        let Ok(entries) = std::fs::read_dir(dir) else {
            return page;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            for (level, heading) in read_headings(&text) {
                let id = slugify(&heading);
                if level == 1 && stem != name {
                    page.methods.insert(stem.to_string(), id.clone());
                }
                if level > 1 || stem != name {
                    page.ids.insert(id);
                }
            }
        }
        page
    }

    /// The level and text of each Markdown heading outside a code fence.
    fn read_headings(text: &str) -> Vec<(usize, String)> {
        let mut headings = Vec::new();
        let mut fenced = false;
        for line in text.lines() {
            if line.trim_start().starts_with("```") {
                fenced = !fenced;
                continue;
            }
            let level = line.chars().take_while(|&c| c == '#').count();
            if !fenced && level > 0 && line[level..].starts_with(' ') {
                headings.push((level, line[level..].trim().to_string()));
            }
        }
        headings
    }

    /// The id github-slugger gives a heading, which is the id Starlight uses.
    fn slugify(heading: &str) -> String {
        heading
            .trim()
            .to_lowercase()
            .chars()
            .filter(|&c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '_'))
            .map(|c| if c == ' ' { '-' } else { c })
            .collect()
    }

    /// Every Markdown file under `dir`, at any depth.
    fn list_markdown(dir: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let Ok(entries) = std::fs::read_dir(dir) else {
            return files;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(list_markdown(&path));
            } else if path.extension().and_then(|s| s.to_str()) == Some("md") {
                files.push(path);
            }
        }
        files.sort();
        files
    }

    /// The part after `/api/` of each fragmentcolor.org/api link in `text`.
    fn find_links(text: &str) -> Vec<&str> {
        let mut links = Vec::new();
        for host in HOSTS {
            for (start, _) in text.match_indices(host) {
                let rest = &text[start + host.len()..];
                let end = rest
                    .find(|c: char| !(c.is_ascii_alphanumeric() || "/_#.-".contains(c)))
                    .unwrap_or(rest.len());
                let link = rest[..end].trim_end_matches(['.', '/']);
                if !link.is_empty() {
                    links.push(link);
                }
            }
        }
        links
    }

    /// Why `link` doesn't reach a page and heading the site builds, with the link to use.
    fn check_link(
        pages: &HashMap<String, Page>,
        dirs: &HashMap<String, String>,
        link: &str,
    ) -> Option<String> {
        let (path, anchor) = link.split_once('#').unwrap_or((link, ""));
        let path = path.trim_end_matches('/');
        if let Some(page) = pages.get(path) {
            if anchor.is_empty() || has_id(page, anchor) {
                return None;
            }
            let hint = page
                .methods
                .get(anchor)
                .map(|id| format!("; use {SITE}{path}#{id}"))
                .unwrap_or_default();
            return Some(format!("{SITE}{link} names no heading on its page{hint}"));
        }
        Some(format!("{SITE}{link} names no page{}", suggest(pages, dirs, path)))
    }

    /// Whether `anchor` is a heading id on `page`, counting the `-1`, `-2` ids of repeats.
    fn has_id(page: &Page, anchor: &str) -> bool {
        if page.ids.contains(anchor) {
            return true;
        }
        anchor
            .rsplit_once('-')
            .is_some_and(|(id, n)| n.parse::<u32>().is_ok() && page.ids.contains(id))
    }

    /// The link to use for a folder path or an `<object>/<method>` path, when there is one.
    fn suggest(
        pages: &HashMap<String, Page>,
        dirs: &HashMap<String, String>,
        path: &str,
    ) -> String {
        if let Some(key) = dirs.get(path) {
            return format!("; use {SITE}{key}");
        }
        let Some((head, method)) = path.rsplit_once('/') else {
            return String::new();
        };
        let key = if pages.contains_key(head) {
            head
        } else {
            match dirs.get(head) {
                Some(key) => key.as_str(),
                None => return String::new(),
            }
        };
        match pages.get(key).and_then(|page| page.methods.get(method)) {
            Some(id) => format!("; use {SITE}{key}#{id}"),
            None => String::new(),
        }
    }
}
