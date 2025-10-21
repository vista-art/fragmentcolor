import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";
import starlightBlog from "starlight-blog";
import path from "node:path";
import { fileURLToPath } from "node:url";

import vercel from "@astrojs/vercel";

// https://astro.build/config
export default defineConfig({
  site: "https://fragmentcolor.org",
  integrations: [
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
              title: "Creator of FragmentColor",
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
            const ensureSidebarLink = (ul, href, text) => {
              try {
                if (!ul) return;
                const alt = href.endsWith('/') ? href.slice(0, -1) : href + '/';
                const existingAnchor = ul.querySelector('a[href="' + href + '"], a[href="' + alt + '"]');
                if (existingAnchor) {
                  const li = existingAnchor.closest('li');
                  if (li && li.parentElement === ul) ul.appendChild(li); // move to last
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
                ul.appendChild(li); // add as last item
              } catch {}
            };

            const wire = () => {
              const isBlog = location.pathname.startsWith('/blog/');
              const uls = document.querySelectorAll('#starlight__sidebar .top-level');
              uls.forEach((ul) => {
                if (isBlog) {
                  // Blog sidebar: add Docs (last)
                  ensureSidebarLink(ul, '/welcome/', 'Docs');
                } else {
                  // Docs (or non-blog) sidebar: add Blog (last)
                  ensureSidebarLink(ul, '/blog/', 'Blog');
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
          label: "API Reference",
          autogenerate: { directory: "api" },
        },
      ],
    }),
  ],

  adapter: vercel(),
});