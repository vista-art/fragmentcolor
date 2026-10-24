// heading-ids: an Astro integration that reads a trailing `{#id}` on a heading.
//
// The API pages build.rs writes end each method's heading with its name,
// as in `### Scene::remove_pass(name: &str) \{#remove_pass\}`, where the
// backslashes keep MDX from reading the braces as an expression. This
// plugin makes `remove_pass` the heading's id and drops it from the text,
// so the table of contents, the heading's own anchor link and every link
// to `/api/scene/scene#remove_pass` share one short id.
//
// Astro renders Markdown and MDX with Sätteri, so this is a Sätteri mdast
// plugin, the counterpart of a remark plugin. The integration adds it to
// the configured processor, the way Starlight adds its own plugins.

import type { AstroIntegration } from "astro";

const PATTERN = /\s*\{#([\w-]+)\}\s*$/;

interface Node {
  type: string;
  value?: string;
  children?: Node[];
  data?: { hProperties?: Record<string, unknown> };
}

interface Context {
  setProperty(node: Node, key: string, value: unknown): void;
  removeNode(node: Node): void;
}

/** Moves a trailing `{#id}` from a heading's text to its id. */
function setHeadingId(heading: Node, context: Context): void {
  const last = heading.children?.at(-1);
  if (last?.type !== "text" || last.value === undefined) return;
  const match = PATTERN.exec(last.value);
  if (!match) return;
  const text = last.value.slice(0, match.index);
  if (text) context.setProperty(last, "value", text);
  else context.removeNode(last);
  const hProperties = { ...heading.data?.hProperties, id: match[1] };
  context.setProperty(heading, "data", { ...heading.data, hProperties });
}

export default function headingIds(): AstroIntegration {
  return {
    name: "heading-ids",
    hooks: {
      "astro:config:setup": ({ config }) => {
        const processor = config.markdown.processor;
        if (processor.name !== "satteri") {
          throw new Error(
            `heading-ids supports the Sätteri Markdown processor, not "${processor.name}".`,
          );
        }
        const options = processor.options as { mdastPlugins: unknown[] };
        options.mdastPlugins.push({ name: "heading-ids", heading: setHeadingId });
      },
    },
  };
}
