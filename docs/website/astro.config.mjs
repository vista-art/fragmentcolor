import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";
import starlightBlog from "starlight-blog";


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