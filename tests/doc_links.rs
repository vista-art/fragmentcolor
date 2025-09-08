use std::collections::HashSet;

use fragmentcolor::doc_link::rewrite_links_to_site_root;

#[test]
pub fn mdx_abs_links_are_root_absolute() {
    let mdx = "See [Renderer](https://fragmentcolor.org/api/core/renderer) and [Shader](https://www.fragmentcolor.org/api/core/shader).";
    let base = "/api";
    let mut cats = HashSet::new();
    cats.insert("core".to_string());

    let out = rewrite_links_to_site_root(mdx, &cats, base);
    assert!(
        out.contains("](/api/core/renderer)"),
        "renderer link not normalized: {}",
        out
    );
    assert!(
        out.contains("](/api/core/shader)"),
        "shader link not normalized: {}",
        out
    );
}

#[test]
pub fn mdx_category_relative_links_are_prefixed() {
    let mdx = "See [Target](core/target) and [WindowTarget](targets/windowtarget).";
    let base = "/api";
    let mut cats = HashSet::new();
    cats.insert("core".to_string());
    cats.insert("targets".to_string());

    let out = rewrite_links_to_site_root(mdx, &cats, base);
    assert!(
        out.contains("](/api/core/target)"),
        "core/target missing prefix: {}",
        out
    );
    assert!(
        out.contains("](/api/targets/windowtarget)"),
        "targets/windowtarget missing prefix: {}",
        out
    );
}

#[test]
pub fn custom_base_is_honored() {
    let mdx =
        "See [Renderer](https://fragmentcolor.org/api/core/renderer) and [Target](core/target).";
    let base = "/docs/api";
    let mut cats = HashSet::new();
    cats.insert("core".to_string());

    let out = rewrite_links_to_site_root(mdx, &cats, base);
    assert!(
        out.contains("](/docs/api/core/renderer)"),
        "abs link base not applied: {}",
        out
    );
    assert!(
        out.contains("](/docs/api/core/target)"),
        "relative link base not applied: {}",
        out
    );
}
