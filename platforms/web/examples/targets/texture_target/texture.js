const renderer = new Renderer();
const target = await renderer.createTextureTarget([512, 512]);
const postShader = new Shader(`
  @group(0) @binding(0) var t: texture_2d<f32>;
  @fragment fn main() -> @location(0) vec4f {
    return textureSample(t, s, in.uv);
  }
`);
const tex = target.texture();
await postShader.set("t", tex);
renderer.render(postShader, target);