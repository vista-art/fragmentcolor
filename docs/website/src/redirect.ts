// Permanent redirects for pages that moved.
// Vercel routes an on-demand endpoint with or without its trailing slash,
// while an entry in the `redirects` config matches only the URL without it.

import type { APIRoute } from "astro";

export function redirectTo(destination: string): APIRoute {
  return ({ redirect }) => redirect(destination, 301);
}
