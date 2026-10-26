// Records compose.webp: adding sdf2d/equilateral_triangle from the library picker, calling it in place
// of the star, and removing sdf2d/star. Redo it with `./fc capture record docs/media/compose.record.mjs`.
export default {
  page: "/#shapes",
  viewportHeight: 760,
  fps: 8,
  quality: 58,
  out: "compose.webp",
  async play(r) {
    await r.wait(0.8);
    await r.tab("shader.wgsl");
    await r.click("fc-playground .fcp-add");
    await r.wait(0.4);
    await r.click(".fcp-picker-search:visible", { duration: 0.35 });
    await r.type("triangle", { delay: 90 });
    await r.wait(0.7);
    await r.click('.fcp-picker-card:visible:has-text("equilateral_triangle")');
    await r.wait(0.5);
    await r.click(".fcp-picker-close:visible");
    await r.reveal("let d_star = star(", { duration: 0.5 });
    await r.caret("= star", { at: "end" });
    await r.press("Backspace", { times: 4, delay: 90 });
    await r.type("equilateral_triangle", { delay: 60 });
    await r.select(", 5u, 3.0");
    await r.press("Backspace");
    await r.press("Home");
    await r.wait(1.3);
    await r.click('fc-playground .fcp-chip:has-text("sdf2d/star") button');
    await r.wait(2.2);
  },
};
