// Auto-generated: runs all JS examples with cargo-like output.
const GREEN='\u001b[1;32m'; const RED='\u001b[1;31m'; const RESET='\u001b[0m';
const EXAMPLES = [
  '../examples/core/frame/Frame.js',
  '../examples/core/frame/add_pass.js',
  '../examples/core/frame/new.js',
  '../examples/core/pass/Pass.js',
  '../examples/core/pass/add_depth_target.js',
  '../examples/core/pass/add_mesh.js',
  '../examples/core/pass/add_mesh_to_shader.js',
  '../examples/core/pass/add_shader.js',
  '../examples/core/pass/add_target.js',
  '../examples/core/pass/compute.js',
  '../examples/core/pass/from_shader.js',
  '../examples/core/pass/get_input.js',
  '../examples/core/pass/is_compute.js',
  '../examples/core/pass/load_previous.js',
  '../examples/core/pass/new.js',
  '../examples/core/pass/require.js',
  '../examples/core/pass/set_clear_color.js',
  '../examples/core/pass/set_compute_dispatch.js',
  '../examples/core/pass/set_viewport.js',
  '../examples/core/renderer/Renderer.js',
  '../examples/core/renderer/create_depth_texture.js',
  '../examples/core/renderer/create_storage_texture.js',
  '../examples/core/renderer/create_target.js',
  '../examples/core/renderer/create_texture.js',
  '../examples/core/renderer/create_texture_target.js',
  '../examples/core/renderer/create_texture_with.js',
  '../examples/core/renderer/create_texture_with_format.js',
  '../examples/core/renderer/create_texture_with_size.js',
  '../examples/core/renderer/new.js',
  '../examples/core/renderer/render.js',
  '../examples/core/shader/Shader.js',
  '../examples/core/shader/add_mesh.js',
  '../examples/core/shader/clear_meshes.js',
  '../examples/core/shader/from_mesh.js',
  '../examples/core/shader/from_vertex.js',
  '../examples/core/shader/get.js',
  '../examples/core/shader/is_compute.js',
  '../examples/core/shader/list_keys.js',
  '../examples/core/shader/list_uniforms.js',
  '../examples/core/shader/new.js',
  '../examples/core/shader/remove_mesh.js',
  '../examples/core/shader/remove_meshes.js',
  '../examples/core/shader/set.js',
  '../examples/core/shader/validate_mesh.js',
  '../examples/core/texture/Texture.js',
  '../examples/core/texture/aspect.js',
  '../examples/core/texture/set_sampler_options.js',
  '../examples/core/texture/size.js',
  '../examples/geometry/mesh/Mesh.js',
  '../examples/geometry/mesh/add_instance.js',
  '../examples/geometry/mesh/add_instances.js',
  '../examples/geometry/mesh/add_vertex.js',
  '../examples/geometry/mesh/add_vertices.js',
  '../examples/geometry/mesh/clear_instance_count.js',
  '../examples/geometry/mesh/clear_instances.js',
  '../examples/geometry/mesh/from_vertices.js',
  '../examples/geometry/mesh/new.js',
  '../examples/geometry/mesh/set_instance_count.js',
  '../examples/geometry/quad/Quad.js',
  '../examples/geometry/quad/get_mesh.js',
  '../examples/geometry/quad/new.js',
  '../examples/geometry/vertex/Vertex.js',
  '../examples/geometry/vertex/create_instance.js',
  '../examples/geometry/vertex/new.js',
  '../examples/geometry/vertex/set.js',
  '../examples/targets/target/Target.js',
  '../examples/targets/target/get_image.js',
  '../examples/targets/target/resize.js',
  '../examples/targets/target/size.js',
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
export async function runExamples() {
  const total = EXAMPLES.length;
  let passed = 0;
  let failed = 0;
  console.log(`running ${total} tests`);
  globalThis.__HC = globalThis.__HC || { currentModule: null };
  for (const rel of EXAMPLES) {
    const name = fq(rel);
    const head = `test ${name} ... `;
    try {
      globalThis.__HC.currentModule = name;
      await import(rel);
      passed++;
      console.log(head + GREEN + 'OK' + RESET);
    } catch (e) {
      failed++;
      console.log(head + RED + 'FAILED' + RESET);
      console.error(e);
    } finally {
      globalThis.__HC.currentModule = null;
    }
  }
  return { passed, failed };
}
