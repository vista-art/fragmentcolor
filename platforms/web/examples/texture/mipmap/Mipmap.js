import { Mipmap, Renderer, TextureFormat } from "fragmentcolor";
const png = new Uint8Array(await (await fetch("/healthcheck/public/favicon.png")).arrayBuffer());

// Imagine `png` came off your asset loader on a worker thread.

// Decode + mipmap generation. Pure CPU; run it wherever you like.
const chain = Mipmap.build(png, TextureFormat.Rgba8UnormSrgb);

// Back on the renderer thread, the upload is just a GPU write.
const renderer = new Renderer();
const texture = await renderer.createTexture(chain);