// The Markdown editor inside DevEditor.astro. The page imports this module
// only when the panel opens, so readers never download CodeMirror.

import { EditorState } from "@codemirror/state";
import { EditorView, basicSetup } from "codemirror";
import { markdown } from "@codemirror/lang-markdown";
import { syntaxHighlighting, defaultHighlightStyle } from "@codemirror/language";
import { dracula } from "thememirror";

export type { EditorView };

export function mountEditor(parent: HTMLElement, doc: string): EditorView {
  const view = new EditorView({
    state: EditorState.create({
      doc,
      extensions: [
        basicSetup,
        markdown(),
        dracula,
        syntaxHighlighting(defaultHighlightStyle),
        EditorView.lineWrapping,
        EditorView.theme({
          "&": { height: "100%" },
          ".cm-scroller": { overflow: "auto" },
        }),
      ],
    }),
    parent,
  });
  view.focus();
  return view;
}
