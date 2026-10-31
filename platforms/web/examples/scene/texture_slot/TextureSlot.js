import { Material, TextureSlot } from "fragmentcolor";

const material = Material.pbr().textureUv(TextureSlot.Occlusion, 1).textureTransform(TextureSlot.Normal, [0.0, 0.0], [4.0, 4.0], 0.0);