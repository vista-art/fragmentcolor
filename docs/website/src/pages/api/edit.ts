// Reads and commits blog posts through GitHub for the editor
// on the deployed site. It answers 404 until EDIT_TOKEN is set.

import type { APIRoute } from "astro";
import { EDIT_TOKEN, GITHUB_TOKEN } from "astro:env/server";
import { handleEdit } from "../../server/edit";

export const prerender = false;

export const ALL: APIRoute = ({ request }) =>
  handleEdit(request, { editToken: EDIT_TOKEN, githubToken: GITHUB_TOKEN });
