import { TextureFormat, TextureMipChain } from "fragmentcolor";

const b64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAIAAACQd1PeAAAADElEQVR4nGM4ceIEAAS0AlkWLoFAAAAAAElFTkSuQmCC";
const pngBytes = Uint8Array.from(atob(b64), c => c.charCodeAt(0));
const chain = TextureMipChain.prepare(pngBytes, TextureFormat.Rgba8UnormSrgb, null);
// level(0) returns the base mip level bytes
const level_zero_bytes = chain.level(0);