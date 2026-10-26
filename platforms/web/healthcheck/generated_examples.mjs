// Auto-generated: runs all JS examples with cargo-like output.
const GREEN='\u001b[1;32m'; const RED='\u001b[1;31m'; const RESET='\u001b[0m';
const EXAMPLES = [
  ['../examples/core/pass/Pass.js', 'docs/api/core/pass/pass.md'],
  ['../examples/core/pass/add.js', 'docs/api/core/pass/add.md'],
  ['../examples/core/pass/add_mesh.js', 'docs/api/core/pass/add_mesh.md'],
  ['../examples/core/pass/add_shader.js', 'docs/api/core/pass/add_shader.md'],
  ['../examples/core/pass/compute.js', 'docs/api/core/pass/compute.md'],
  ['../examples/core/pass/from_shader.js', 'docs/api/core/pass/from_shader.md'],
  ['../examples/core/pass/is_compute.js', 'docs/api/core/pass/is_compute.md'],
  ['../examples/core/pass/load_previous.js', 'docs/api/core/pass/load_previous.md'],
  ['../examples/core/pass/name.js', 'docs/api/core/pass/name.md'],
  ['../examples/core/pass/new.js', 'docs/api/core/pass/new.md'],
  ['../examples/core/pass/require.js', 'docs/api/core/pass/require.md'],
  ['../examples/core/pass/set_clear_color.js', 'docs/api/core/pass/set_clear_color.md'],
  ['../examples/core/pass/set_compute_dispatch.js', 'docs/api/core/pass/set_compute_dispatch.md'],
  ['../examples/core/pass/set_depth_target.js', 'docs/api/core/pass/set_depth_target.md'],
  ['../examples/core/pass/set_target.js', 'docs/api/core/pass/set_target.md'],
  ['../examples/core/pass/set_viewport.js', 'docs/api/core/pass/set_viewport.md'],
  ['../examples/core/renderer/Renderer.js', 'docs/api/core/renderer/renderer.md'],
  ['../examples/core/renderer/create_depth_texture.js', 'docs/api/core/renderer/create_depth_texture.md'],
  ['../examples/core/renderer/create_external_texture.js', 'docs/api/core/renderer/create_external_texture.md'],
  ['../examples/core/renderer/create_storage_texture.js', 'docs/api/core/renderer/create_storage_texture.md'],
  ['../examples/core/renderer/create_target.js', 'docs/api/core/renderer/create_target.md'],
  ['../examples/core/renderer/create_texture.js', 'docs/api/core/renderer/create_texture.md'],
  ['../examples/core/renderer/create_texture_target.js', 'docs/api/core/renderer/create_texture_target.md'],
  ['../examples/core/renderer/load.js', 'docs/api/core/renderer/load.md'],
  ['../examples/core/renderer/new.js', 'docs/api/core/renderer/new.md'],
  ['../examples/core/renderer/read_storage.js', 'docs/api/core/renderer/read_storage.md'],
  ['../examples/core/renderer/read_texture.js', 'docs/api/core/renderer/read_texture.md'],
  ['../examples/core/renderer/render.js', 'docs/api/core/renderer/render.md'],
  ['../examples/core/renderer/unregister_texture.js', 'docs/api/core/renderer/unregister_texture.md'],
  ['../examples/core/shader/Shader.js', 'docs/api/core/shader/shader.md'],
  ['../examples/core/shader/add_mesh.js', 'docs/api/core/shader/add_mesh.md'],
  ['../examples/core/shader/clear_meshes.js', 'docs/api/core/shader/clear_meshes.md'],
  ['../examples/core/shader/default.js', 'docs/api/core/shader/default.md'],
  ['../examples/core/shader/fetch.js', 'docs/api/core/shader/fetch.md'],
  ['../examples/core/shader/from_mesh.js', 'docs/api/core/shader/from_mesh.md'],
  ['../examples/core/shader/from_vertex.js', 'docs/api/core/shader/from_vertex.md'],
  ['../examples/core/shader/get.js', 'docs/api/core/shader/get.md'],
  ['../examples/core/shader/is_compute.js', 'docs/api/core/shader/is_compute.md'],
  ['../examples/core/shader/list_keys.js', 'docs/api/core/shader/list_keys.md'],
  ['../examples/core/shader/list_uniforms.js', 'docs/api/core/shader/list_uniforms.md'],
  ['../examples/core/shader/new.js', 'docs/api/core/shader/new.md'],
  ['../examples/core/shader/remove_mesh.js', 'docs/api/core/shader/remove_mesh.md'],
  ['../examples/core/shader/remove_meshes.js', 'docs/api/core/shader/remove_meshes.md'],
  ['../examples/core/shader/set.js', 'docs/api/core/shader/set.md'],
  ['../examples/core/shader/set_registry.js', 'docs/api/core/shader/set_registry.md'],
  ['../examples/core/shader/validate_mesh.js', 'docs/api/core/shader/validate_mesh.md'],
  ['../examples/geometry/instance/Instance.js', 'docs/api/geometry/instance/instance.md'],
  ['../examples/geometry/instance/new.js', 'docs/api/geometry/instance/new.md'],
  ['../examples/geometry/instance/set.js', 'docs/api/geometry/instance/set.md'],
  ['../examples/geometry/mesh/Mesh.js', 'docs/api/geometry/mesh/mesh.md'],
  ['../examples/geometry/mesh/add_instance.js', 'docs/api/geometry/mesh/add_instance.md'],
  ['../examples/geometry/mesh/add_instances.js', 'docs/api/geometry/mesh/add_instances.md'],
  ['../examples/geometry/mesh/add_vertex.js', 'docs/api/geometry/mesh/add_vertex.md'],
  ['../examples/geometry/mesh/add_vertices.js', 'docs/api/geometry/mesh/add_vertices.md'],
  ['../examples/geometry/mesh/clear_indices.js', 'docs/api/geometry/mesh/clear_indices.md'],
  ['../examples/geometry/mesh/clear_instances.js', 'docs/api/geometry/mesh/clear_instances.md'],
  ['../examples/geometry/mesh/from_vertices.js', 'docs/api/geometry/mesh/from_vertices.md'],
  ['../examples/geometry/mesh/new.js', 'docs/api/geometry/mesh/new.md'],
  ['../examples/geometry/mesh/set_indices.js', 'docs/api/geometry/mesh/set_indices.md'],
  ['../examples/geometry/mesh/set_instance_count.js', 'docs/api/geometry/mesh/set_instance_count.md'],
  ['../examples/geometry/quad/Quad.js', 'docs/api/geometry/quad/quad.md'],
  ['../examples/geometry/quad/get_mesh.js', 'docs/api/geometry/quad/get_mesh.md'],
  ['../examples/geometry/quad/new.js', 'docs/api/geometry/quad/new.md'],
  ['../examples/geometry/vertex/Vertex.js', 'docs/api/geometry/vertex/vertex.md'],
  ['../examples/geometry/vertex/create_instance.js', 'docs/api/geometry/vertex/create_instance.md'],
  ['../examples/geometry/vertex/new.js', 'docs/api/geometry/vertex/new.md'],
  ['../examples/geometry/vertex/pbr.js', 'docs/api/geometry/vertex/pbr.md'],
  ['../examples/geometry/vertex/set.js', 'docs/api/geometry/vertex/set.md'],
  ['../examples/scene/camera/look_at.js', 'docs/api/scene/camera/look_at.md'],
  ['../examples/scene/camera/orthographic.js', 'docs/api/scene/camera/orthographic.md'],
  ['../examples/scene/camera/perspective.js', 'docs/api/scene/camera/perspective.md'],
  ['../examples/scene/camera/position.js', 'docs/api/scene/camera/position.md'],
  ['../examples/scene/camera/set_aspect.js', 'docs/api/scene/camera/set_aspect.md'],
  ['../examples/scene/camera/view_proj.js', 'docs/api/scene/camera/view_proj.md'],
  ['../examples/scene/light/color.js', 'docs/api/scene/light/color.md'],
  ['../examples/scene/light/direction.js', 'docs/api/scene/light/direction.md'],
  ['../examples/scene/light/directional.js', 'docs/api/scene/light/directional.md'],
  ['../examples/scene/light/inner_cone_angle.js', 'docs/api/scene/light/inner_cone_angle.md'],
  ['../examples/scene/light/intensity.js', 'docs/api/scene/light/intensity.md'],
  ['../examples/scene/light/kind.js', 'docs/api/scene/light/kind.md'],
  ['../examples/scene/light/outer_cone_angle.js', 'docs/api/scene/light/outer_cone_angle.md'],
  ['../examples/scene/light/point.js', 'docs/api/scene/light/point.md'],
  ['../examples/scene/light/position.js', 'docs/api/scene/light/position.md'],
  ['../examples/scene/light/range.js', 'docs/api/scene/light/range.md'],
  ['../examples/scene/light/set_color.js', 'docs/api/scene/light/set_color.md'],
  ['../examples/scene/light/set_cone_angles.js', 'docs/api/scene/light/set_cone_angles.md'],
  ['../examples/scene/light/set_direction.js', 'docs/api/scene/light/set_direction.md'],
  ['../examples/scene/light/set_intensity.js', 'docs/api/scene/light/set_intensity.md'],
  ['../examples/scene/light/set_position.js', 'docs/api/scene/light/set_position.md'],
  ['../examples/scene/light/set_range.js', 'docs/api/scene/light/set_range.md'],
  ['../examples/scene/light/spot.js', 'docs/api/scene/light/spot.md'],
  ['../examples/scene/material/Material.js', 'docs/api/scene/material/material.md'],
  ['../examples/scene/material/alpha_cutoff.js', 'docs/api/scene/material/alpha_cutoff.md'],
  ['../examples/scene/material/alpha_mode.js', 'docs/api/scene/material/alpha_mode.md'],
  ['../examples/scene/material/base_color.js', 'docs/api/scene/material/base_color.md'],
  ['../examples/scene/material/base_color_texture.js', 'docs/api/scene/material/base_color_texture.md'],
  ['../examples/scene/material/custom.js', 'docs/api/scene/material/custom.md'],
  ['../examples/scene/material/double_sided.js', 'docs/api/scene/material/double_sided.md'],
  ['../examples/scene/material/emissive.js', 'docs/api/scene/material/emissive.md'],
  ['../examples/scene/material/emissive_texture.js', 'docs/api/scene/material/emissive_texture.md'],
  ['../examples/scene/material/metallic.js', 'docs/api/scene/material/metallic.md'],
  ['../examples/scene/material/metallic_roughness_texture.js', 'docs/api/scene/material/metallic_roughness_texture.md'],
  ['../examples/scene/material/normal_scale.js', 'docs/api/scene/material/normal_scale.md'],
  ['../examples/scene/material/normal_texture.js', 'docs/api/scene/material/normal_texture.md'],
  ['../examples/scene/material/occlusion_strength.js', 'docs/api/scene/material/occlusion_strength.md'],
  ['../examples/scene/material/occlusion_texture.js', 'docs/api/scene/material/occlusion_texture.md'],
  ['../examples/scene/material/pbr.js', 'docs/api/scene/material/pbr.md'],
  ['../examples/scene/material/roughness.js', 'docs/api/scene/material/roughness.md'],
  ['../examples/scene/material/shader.js', 'docs/api/scene/material/shader.md'],
  ['../examples/scene/material/texture_transform.js', 'docs/api/scene/material/texture_transform.md'],
  ['../examples/scene/material/texture_uv.js', 'docs/api/scene/material/texture_uv.md'],
  ['../examples/scene/material/uv_transform.js', 'docs/api/scene/material/uv_transform.md'],
  ['../examples/scene/model/material.js', 'docs/api/scene/model/material.md'],
  ['../examples/scene/model/mesh.js', 'docs/api/scene/model/mesh.md'],
  ['../examples/scene/model/new.js', 'docs/api/scene/model/new.md'],
  ['../examples/scene/model/rotate.js', 'docs/api/scene/model/rotate.md'],
  ['../examples/scene/model/scale.js', 'docs/api/scene/model/scale.md'],
  ['../examples/scene/model/set_transform.js', 'docs/api/scene/model/set_transform.md'],
  ['../examples/scene/model/set_visible.js', 'docs/api/scene/model/set_visible.md'],
  ['../examples/scene/model/transform.js', 'docs/api/scene/model/transform.md'],
  ['../examples/scene/model/translate.js', 'docs/api/scene/model/translate.md'],
  ['../examples/scene/model/visible.js', 'docs/api/scene/model/visible.md'],
  ['../examples/scene/scene/add.js', 'docs/api/scene/scene/add.md'],
  ['../examples/scene/scene/add_pass.js', 'docs/api/scene/scene/add_pass.md'],
  ['../examples/scene/scene/add_to.js', 'docs/api/scene/scene/add_to.md'],
  ['../examples/scene/scene/ambient.js', 'docs/api/scene/scene/ambient.md'],
  ['../examples/scene/scene/cameras.js', 'docs/api/scene/scene/cameras.md'],
  ['../examples/scene/scene/find_pass.js', 'docs/api/scene/scene/find_pass.md'],
  ['../examples/scene/scene/get_pass.js', 'docs/api/scene/scene/get_pass.md'],
  ['../examples/scene/scene/lights.js', 'docs/api/scene/scene/lights.md'],
  ['../examples/scene/scene/list_passes.js', 'docs/api/scene/scene/list_passes.md'],
  ['../examples/scene/scene/load.js', 'docs/api/scene/scene/load.md'],
  ['../examples/scene/scene/models.js', 'docs/api/scene/scene/models.md'],
  ['../examples/scene/scene/new.js', 'docs/api/scene/scene/new.md'],
  ['../examples/scene/scene/no_default_camera.js', 'docs/api/scene/scene/no_default_camera.md'],
  ['../examples/scene/scene/no_default_light.js', 'docs/api/scene/scene/no_default_light.md'],
  ['../examples/scene/scene/no_defaults.js', 'docs/api/scene/scene/no_defaults.md'],
  ['../examples/scene/scene/remove_pass.js', 'docs/api/scene/scene/remove_pass.md'],
  ['../examples/scene/scene/set_default_camera.js', 'docs/api/scene/scene/set_default_camera.md'],
  ['../examples/scene/scene/set_default_light.js', 'docs/api/scene/scene/set_default_light.md'],
  ['../examples/scene/scene/set_passes.js', 'docs/api/scene/scene/set_passes.md'],
  ['../examples/scene/texture_slot/TextureSlot.js', 'docs/api/scene/texture_slot/texture_slot.md'],
  ['../examples/targets/target/Target.js', 'docs/api/targets/target/target.md'],
  ['../examples/targets/target/get_image.js', 'docs/api/targets/target/get_image.md'],
  ['../examples/targets/target/resize.js', 'docs/api/targets/target/resize.md'],
  ['../examples/targets/target/size.js', 'docs/api/targets/target/size.md'],
  ['../examples/targets/texture_target/TextureTarget.js', 'docs/api/targets/texture_target/texture_target.md'],
  ['../examples/targets/texture_target/get_image.js', 'docs/api/targets/texture_target/get_image.md'],
  ['../examples/targets/texture_target/resize.js', 'docs/api/targets/texture_target/resize.md'],
  ['../examples/targets/texture_target/size.js', 'docs/api/targets/texture_target/size.md'],
  ['../examples/targets/texture_target/texture.js', 'docs/api/targets/texture_target/texture.md'],
  ['../examples/targets/window_target/WindowTarget.js', 'docs/api/targets/window_target/window_target.md'],
  ['../examples/targets/window_target/get_image.js', 'docs/api/targets/window_target/get_image.md'],
  ['../examples/targets/window_target/resize.js', 'docs/api/targets/window_target/resize.md'],
  ['../examples/targets/window_target/size.js', 'docs/api/targets/window_target/size.md'],
  ['../examples/texture/mipmap/Mipmap.js', 'docs/api/texture/mipmap/mipmap.md'],
  ['../examples/texture/mipmap/build.js', 'docs/api/texture/mipmap/build.md'],
  ['../examples/texture/mipmap/count.js', 'docs/api/texture/mipmap/count.md'],
  ['../examples/texture/mipmap/format.js', 'docs/api/texture/mipmap/format.md'],
  ['../examples/texture/mipmap/level.js', 'docs/api/texture/mipmap/level.md'],
  ['../examples/texture/mipmap/levels.js', 'docs/api/texture/mipmap/levels.md'],
  ['../examples/texture/mipmap/size.js', 'docs/api/texture/mipmap/size.md'],
  ['../examples/texture/texture/Texture.js', 'docs/api/texture/texture/texture.md'],
  ['../examples/texture/texture/aspect.js', 'docs/api/texture/texture/aspect.md'],
  ['../examples/texture/texture/get_image.js', 'docs/api/texture/texture/get_image.md'],
  ['../examples/texture/texture/id.js', 'docs/api/texture/texture/id.md'],
  ['../examples/texture/texture/resize.js', 'docs/api/texture/texture/resize.md'],
  ['../examples/texture/texture/set_sampler_options.js', 'docs/api/texture/texture/set_sampler_options.md'],
  ['../examples/texture/texture/size.js', 'docs/api/texture/texture/size.md'],
  ['../examples/texture/texture/write.js', 'docs/api/texture/texture/write.md'],
  ['../examples/texture/texture/write_region.js', 'docs/api/texture/texture/write_region.md'],
]

function fq(rel){ return 'platforms.web.examples.' + rel.replace('../examples/','').replace(/\\.js$/, '').replaceAll('/', '.'); }

// The error scopes WebGPU has yet to resolve.
const pending = new Set();

// Counts each popErrorScope until its promise settles.
function trackScopes() {
  const device = globalThis.GPUDevice;
  if (!device || device.prototype.popErrorScope.__fcTracked) return;
  const pop = device.prototype.popErrorScope;
  const tracked = function () {
    const promise = pop.call(this);
    pending.add(promise);
    const forget = () => pending.delete(promise);
    promise.then(forget, forget);
    return promise;
  };
  tracked.__fcTracked = true;
  device.prototype.popErrorScope = tracked;
}

// Waits until every open scope has resolved and the library has logged what it caught.
// The rounds are capped, so an example that keeps rendering can't hold the run.
async function settleScopes() {
  for (let round = 0; round < 100 && pending.size > 0; round++) {
    await Promise.allSettled([...pending]);
  }
  await new Promise((resolve) => setTimeout(resolve, 0));
}

export async function runExamples() {
  const total = EXAMPLES.length;
  let passed = 0;
  let failed = 0;
  console.log(`running ${total} tests`);
  globalThis.__HC = globalThis.__HC || { currentModule: null };
  trackScopes();
  const consoleError = console.error;
  let logged = [];
  console.error = (...args) => {
    logged.push(args.map(String).join(' '));
    consoleError.apply(console, args);
  };
  try {
    for (const [rel, page] of EXAMPLES) {
      const name = fq(rel);
      const head = `test ${name} ... `;
      logged = [];
      let problem = null;
      try {
        globalThis.__HC.currentModule = name;
        await import(rel);
      } catch (e) {
        problem = e;
      }
      await settleScopes();
      globalThis.__HC.currentModule = null;
      if (problem === null && logged.length > 0) problem = logged.join('\n');
      if (problem === null) {
        passed++;
        console.log(head + GREEN + 'OK' + RESET);
      } else {
        failed++;
        console.log(head + RED + 'FAILED' + RESET);
        console.error(`${page}:`, problem);
      }
    }
  } finally {
    console.error = consoleError;
  }
  return { passed, failed };
}
