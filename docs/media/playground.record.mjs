// Records playground.webp: picking a demo from the cards, a shader edit that rebuilds as it is typed,
// a generated slider, and the Rust tab. Redo it with `./fc capture record docs/media/playground.record.mjs`.
export default {
  page: "/#shapes",
  viewportHeight: 760,
  fps: 8,
  quality: 55,
  out: "playground.webp",
  async play(r) {
    await r.wait(0.8);
    await r.click("fc-playground .fcp-menu-button");
    await r.wait(1.1);
    await r.click('.fcp-demo-picker .fcp-picker-card:has-text("Cells")');
    await r.wait(1.1);
    await r.tab("shader.wgsl");
    await r.reveal("f.x * 0.8 + t");
    await r.select("0.8");
    await r.type("3.5", { delay: 260 });
    await r.wait(1.2);
    await r.slide("u.scale", 6);
    await r.wait(0.8);
    await r.tab("Rust");
    await r.wait(2.2);
  },
};
