import { Renderer, TextureFormat, TextureMipChain } from "fragmentcolor";

// Encoded path: bytes are decoded as an image (PNG/JPEG/etc.), size=null.
const b64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAIAAACQd1PeAAAADElEQVR4nGM4ceIEAAS0AlkWLoFAAAAAAElFTkSuQmCC";
const encodedPngBytes = Uint8Array.from(atob(b64), c => c.charCodeAt(0));
const chain = TextureMipChain.prepare(encodedPngBytes, TextureFormat.Rgba8UnormSrgb, null);

// Hand the chain to the unified create_texture entry.
const renderer = new Renderer();
const texture = await renderer.createTexture(chain);
