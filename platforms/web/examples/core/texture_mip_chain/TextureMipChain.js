import { Renderer, TextureFormat, TextureMipChain } from "fragmentcolor";

const renderer = new Renderer();
// Minimal 1x1 encoded PNG bytes.
const b64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAIAAACQd1PeAAAADElEQVR4nGM4ceIEAAS0AlkWLoFAAAAAAElFTkSuQmCC";
const pngBytes = Uint8Array.from(atob(b64), c => c.charCodeAt(0));
// Encoded path: size=null, bytes decoded as an image.
const chain = TextureMipChain.prepare(pngBytes, TextureFormat.Rgba8UnormSrgb, null);

// Hand the chain to the unified create_texture entry - same vocabulary as
// every other texture path; From<TextureMipChain> selects the GPU-only
// upload internally.
const texture = await renderer.createTexture(chain);