import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";
import starlightBlog from "starlight-blog";
import path from "node:path";
import { fileURLToPath } from "node:url";

import vercel from "@astrojs/vercel";

// Lock-block versioning: scans `<Lock id="...">` regions in MDX/MD on
// dev-server start + on every save (via Vite's watcher), and on every
// production build. Owner of `.claude/locks/locks.json`. See the
// integration source for full docs.
import locks from "./integrations/locks";

// https://astro.build/config
export default defineConfig({
  site: "https://fragmentcolor.org",
  integrations: [
    locks(),
    starlight({
      title: "Fragment Color",
      logo: {
        dark: "./src/assets/logo.svg",
        light: "./src/assets/logo-black.svg",
        replacesTitle: true,
      },
      favicon: "./favicon.svg",
      components: {
        Repl: "./src/components/Repl.astro",
        Examples: "./src/components/Examples.mdx",
        SiteTitle: "./src/components/SiteTitle.astro",
        Footer: "./src/components/Footer.astro",
      },
      customCss: ["./src/assets/styles/override.css"],
      // @TODO set up a system for automatic translation
      //       and config it following these instructions:
      //       https://starlight.astro.build/guides/i18n/
      // // Set English as the default language for this site.
      // defaultLocale: "root",
      // locales: {
      //   // English docs in `src/content/docs/en/`
      //   root: {
      //     label: "English",
      //     lang: "en",
      //   },
      //   // Brazilian Portuguese docs in `src/content/docs/pt-br/`
      //   "pt-br": {
      //     label: "Português Brasileiro",
      //     lang: "pt-BR",
      //   },
      //   // European Portuguese docs in `src/content/docs/pt-pt/`
      //   "pt-pt": {
      //     label: "Português Europeu",
      //     lang: "pt-PT",
      //   },
      //   // German docs in `src/content/docs/ar/`
      //   de: {
      //     label: "العربية",
      //     dir: "rtl",
      //   },
      // },
      plugins: [
        starlightBlog({
          authors: {
            rafaelbeckel: {
              name: "Rafael Beckel",
              title: "Maintainer",
              picture: "/favicon.png", // Images in the `public` directory are supported.
              url: "https://github.com/rafaelbeckel",
            },
          },
        }),
      ],
      head: [
        // Add ICO favicon fallback for Safari.
        {
          tag: "link",
          attrs: {
            rel: "icon",
            href: "./favicon.ico",
          },
        },
        // Cross-link blog and docs inside the left sidebar (desktop & mobile)
        {
          tag: "script",
          attrs: { type: "module" },
          content: `
            const ensureSidebarLink = (ul, href, text, position) => {
              try {
                if (!ul) return;
                const alt = href.endsWith('/') ? href.slice(0, -1) : href + '/';
                const existingAnchor = ul.querySelector('a[href="' + href + '"], a[href="' + alt + '"]');
                if (existingAnchor) {
                  const li = existingAnchor.closest('li');
                  if (li && li.parentElement === ul) {
                    if (position === 'first') ul.insertBefore(li, ul.firstChild);
                    else ul.appendChild(li);
                  }
                  return;
                }
                // Try to clone a simple top-level <li> if present to preserve styling
                let liTemplate = ul.querySelector(':scope > li > a')?.parentElement;
                let li;
                if (liTemplate) {
                  li = liTemplate.cloneNode(true);
                  const a = li.querySelector('a');
                  if (a) {
                    a.href = href;
                    a.textContent = text;
                    // Remove special mobile-only class if present
                    a.classList.remove('sl-blog-mobile-link');
                    // The template might be the currently-active page (e.g. "All Posts"
                    // is the first top-level <li> on /blog/). Strip any active state
                    // markers so the injected cross-link doesn't show up as selected.
                    a.removeAttribute('aria-current');
                    a.classList.remove('sl-link-current');
                    li.removeAttribute('data-current-parent');
                    li.classList.remove('sl-link-current', 'current');
                  } else {
                    li.innerHTML = '';
                    const na = document.createElement('a');
                    na.href = href; na.textContent = text; na.classList.add('large');
                    li.appendChild(na);
                  }
                } else {
                  li = document.createElement('li');
                  const a = document.createElement('a');
                  a.href = href; a.textContent = text; a.classList.add('large');
                  li.appendChild(a);
                }
                if (position === 'first') ul.insertBefore(li, ul.firstChild);
                else ul.appendChild(li);
              } catch {}
            };

            const wire = () => {
              const isBlog = location.pathname.startsWith('/blog/');
              const uls = document.querySelectorAll('#starlight__sidebar .top-level');
              uls.forEach((ul) => {
                if (isBlog) {
                  // Blog sidebar: pin Docs at the top so it stays visible even
                  // when Recent Posts and Tags are expanded and push everything down.
                  ensureSidebarLink(ul, '/welcome/', 'Docs', 'first');
                } else {
                  // Docs (or non-blog) sidebar: add Blog last as a sibling section.
                  ensureSidebarLink(ul, '/blog/', 'Blog', 'last');
                }
              });
            };

            if (document.readyState === 'loading') {
              document.addEventListener('DOMContentLoaded', wire);
            } else {
              wire();
            }
          `,
        },
        // <Lang /> wiring — keeps inline tab-synced text up to date
        // without rendering DOM siblings next to the substitution span.
        // Lives here (page-level head) so the component itself only
        // emits the <span>, avoiding inline-context whitespace artefacts.
        {
          tag: "style",
          content: ".lang-text{display:inline;margin:0;padding:0;white-space:normal;}",
        },
        {
          tag: "script",
          attrs: { type: "module" },
          content: `
            const STORAGE_KEY = "starlight-synced-tabs__lang";
            const LABEL_TO_DATA = {
              Rust: "rust",
              JavaScript: "js",
              Python: "py",
              Swift: "swift",
              Kotlin: "kotlin",
            };
            function updateLangText() {
              let label;
              try { label = localStorage.getItem(STORAGE_KEY); } catch {}
              const key = LABEL_TO_DATA[label || ""] || "rust";
              document.querySelectorAll(".lang-text").forEach((el) => {
                const v = el.dataset[key];
                if (v !== undefined && v !== null) el.textContent = v;
              });
            }
            function initLangText() {
              updateLangText();
              document.addEventListener("click", (e) => {
                const tab = e.target?.closest?.('[role="tab"]');
                if (!tab) return;
                const tabsParent = tab.closest('starlight-tabs[data-sync-key="lang"]');
                if (!tabsParent) return;
                requestAnimationFrame(updateLangText);
              });
            }
            if (document.readyState === "loading") {
              document.addEventListener("DOMContentLoaded", initLangText);
            } else {
              initLangText();
            }
          `,
        },
      ],
      social: [
        { icon: "github", label: "GitHub", href: "https://github.com/vista-art/fragmentcolor" },
      ],
      sidebar: [
        {
          label: "Welcome",
          items: [
            // Each item here is one entry in the navigation menu.
            { label: "Start Here", link: "/welcome" },
            { label: "Platform Support", link: "/welcome/platforms" },
            // { label: "Playground", link: "/welcome/playground" }, // @TODO uncomment when JS integration is ready
          ],
        },
        {
          label: "Tutorials",
          autogenerate: { directory: "tutorials" },
        },
        {
          label: "Shader Catalog",
          autogenerate: { directory: "shaders" },
          collapsed: true,
        },
        {
          label: "API Reference",
          autogenerate: { directory: "api" },
        },
      ],
    }),
  ],

  adapter: vercel(),

  vite: {
    optimizeDeps: {
      // These packages break Vite's dep pre-bundler in this Astro+Vite combo:
      // optimize step claims success but the bundled file never lands in
      // node_modules/.vite/deps, so requests hit "504 Outdated Optimize Dep"
      // with empty body — Firefox surfaces it as MIME-type / nosniff errors.
      // Excluding them serves the packages as native ESM at runtime.
      exclude: [
        "fragmentcolor",
        "codemirror",
        "@codemirror/state",
        "@codemirror/language",
        "@codemirror/lang-markdown",
        "thememirror",
      ],
    },
  },
});
