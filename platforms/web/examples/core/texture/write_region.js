import { Renderer, TextureFormat } from "fragmentcolor";

const renderer = new Renderer();
const texture = await renderer.createStorageTexture([64, 32], TextureFormat.Rgba, null);
const bytes = new Uint8Array(64 * 32 * 4);

// Simple sub-rectangle update.
texture.writeRegion(bytes, [0, 0, 64, 32]);

// Explicit data layout (advanced).
texture.writeRegion(bytes, { x: 0, y: 0, width: 64, height: 32, bytesPerRow: 256, rowsPerImage: 32 });