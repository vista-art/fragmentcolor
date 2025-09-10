// Auto-generated: runs all JS examples with cargo-like output.
const GREEN='\u001b[1;32m'; const RED='\u001b[1;31m'; const RESET='\u001b[0m';
const EXAMPLES = [
  '../examples/core/frame/Frame.js',
  '../examples/core/frame/add_pass.js',
  '../examples/core/frame/new.js',
  '../examples/core/pass/Pass.js',
  '../examples/core/pass/add_shader.js',
  '../examples/core/pass/from_shader.js',
  '../examples/core/pass/new.js',
  '../examples/core/pass/set_clear_color.js',
  '../examples/core/renderer/Renderer.js',
  '../examples/core/renderer/create_target.js',
  '../examples/core/renderer/create_texture_target.js',
  '../examples/core/renderer/new.js',
  '../examples/core/renderer/render.js',
  '../examples/core/shader/Shader.js',
  '../examples/core/shader/get.js',
  '../examples/core/shader/list_keys.js',
  '../examples/core/shader/list_uniforms.js',
  '../examples/core/shader/new.js',
  '../examples/core/shader/set.js',
  '../examples/core/target/Target.js',
  '../examples/core/target/get_current_frame.js',
  '../examples/core/target/get_image.js',
  '../examples/core/target/resize.js',
  '../examples/core/target/size.js',
  '../examples/targets/texture_target/TextureTarget.js',
  '../examples/targets/texture_target/get_image.js',
  '../examples/targets/texture_target/resize.js',
  '../examples/targets/texture_target/size.js',
  '../examples/targets/window_target/WindowTarget.js',
  '../examples/targets/window_target/get_image.js',
  '../examples/targets/window_target/resize.js',
  '../examples/targets/window_target/size.js',
]

function fq(rel){ return 'platforms.web.examples.' + rel.replace('../examples/','').replace(/\\.js$/, '').replaceAll('/', '.'); }
(async () => {
  let failed = 0;
  for (const rel of EXAMPLES) {
    const name = fq(rel);
    const head = `test ${name} ... `;
    try {
      await import(rel);
      console.log(head + GREEN + 'OK' + RESET);
    } catch (e) {
      failed++;
      console.log(head + RED + 'FAILED' + RESET);
      console.error(e);
    }
  }
  if (failed === 0) {
    console.log('Headless JS render completed successfully');
  } else {
    throw new Error(`${failed} JS examples failed`);
  }
})();
