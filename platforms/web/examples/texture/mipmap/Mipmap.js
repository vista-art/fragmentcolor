import { Renderer, TextureFormat, Mipmap } from "fragmentcolor";

const renderer = new Renderer();
// Real encoded PNG bytes: served by the healthcheck server so the example
// runs end-to-end without packaging its own fixture.
const pngResp = await fetch("/healthcheck/public/favicon.png");
const pngBytes = new Uint8Array(await pngResp.arrayBuffer());
const chain = Mipmap.build(pngBytes, TextureFormat.Rgba8UnormSrgb);

// Upload the chain through the regular createTexture entry point.
const texture = await renderer.createTexture(chain);
const _ = texture.size();