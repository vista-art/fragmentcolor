# Tutorials

Rules for every tutorial: its code under `docs/tutorials/<slug>/` and its page under
`docs/website/src/content/docs/tutorials/`. A tutorial's own `AGENTS.md` adds rules for its steps,
and `PLANNED_TUTORIALS.md` holds the order and the conventions for slugs, sidebar order and drafts.

## Shaders and illustrations

A tutorial teaches with two kinds of pictures, and the illustrations enrich the text between the shaders.

- A live shader (`<Demo>`) appears where the page shows code that renders, and again wherever a step changes that code.
- An illustration appears beside the text that explains a concept. It shows what the words alone can't:
  the coordinate system, how rasterization turns a triangle into pixels, what a distance field measures,
  how a buffer moves between passes.

An illustration is a diagram, or a capture of the canvas with arrows, circles, highlights and labels drawn
on top that point at what the shader is doing. Each one sits next to the paragraph it explains, follows the
site's light and dark themes, reads at phone width, and has alt text that says what it shows. A capture can
be redone from its step when the code changes, so a tool makes it, never a hand-edited screenshot.
