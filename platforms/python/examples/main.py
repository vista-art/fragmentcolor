# Auto-generated: executes all Python examples with cargo-like output.
import runpy, pathlib, sys, traceback, os, tempfile

GREEN='\x1b[1;32m'
RED='\x1b[1;31m'
RESET='\x1b[0m'

def gpu_error_log():
    """The file where FragmentColor appends each GPU error that no error scope caught."""
    path = os.environ.get('FC_GPU_ERROR_LOG')
    if not path:
        handle, path = tempfile.mkstemp(prefix='fc-gpu-errors-', suffix='.log')
        os.close(handle)
        os.environ['FC_GPU_ERROR_LOG'] = path
    return path

def take_gpu_errors():
    """Empties the GPU error log and returns what it held."""
    with open(gpu_error_log(), 'a+') as log:
        log.seek(0)
        errors = log.read().strip()
        log.truncate(0)
    return errors

def run_all():
    base = pathlib.Path(__file__).parent
    examples = [
        ('core/pass/Pass.py', 'docs/api/core/pass/pass.md'),
        ('core/pass/add.py', 'docs/api/core/pass/add.md'),
        ('core/pass/add_mesh.py', 'docs/api/core/pass/add_mesh.md'),
        ('core/pass/add_shader.py', 'docs/api/core/pass/add_shader.md'),
        ('core/pass/compute.py', 'docs/api/core/pass/compute.md'),
        ('core/pass/from_shader.py', 'docs/api/core/pass/from_shader.md'),
        ('core/pass/get_input.py', 'docs/api/core/pass/get_input.md'),
        ('core/pass/is_compute.py', 'docs/api/core/pass/is_compute.md'),
        ('core/pass/load_previous.py', 'docs/api/core/pass/load_previous.md'),
        ('core/pass/name.py', 'docs/api/core/pass/name.md'),
        ('core/pass/new.py', 'docs/api/core/pass/new.md'),
        ('core/pass/require.py', 'docs/api/core/pass/require.md'),
        ('core/pass/set_clear_color.py', 'docs/api/core/pass/set_clear_color.md'),
        ('core/pass/set_compute_dispatch.py', 'docs/api/core/pass/set_compute_dispatch.md'),
        ('core/pass/set_depth_target.py', 'docs/api/core/pass/set_depth_target.md'),
        ('core/pass/set_target.py', 'docs/api/core/pass/set_target.md'),
        ('core/pass/set_viewport.py', 'docs/api/core/pass/set_viewport.md'),
        ('core/pass_input/PassInput.py', 'docs/api/core/pass_input/pass_input.md'),
        ('core/pass_input/color.py', 'docs/api/core/pass_input/color.md'),
        ('core/pass_input/loads_previous.py', 'docs/api/core/pass_input/loads_previous.md'),
        ('core/renderer/Renderer.py', 'docs/api/core/renderer/renderer.md'),
        ('core/renderer/create_depth_texture.py', 'docs/api/core/renderer/create_depth_texture.md'),
        ('core/renderer/create_external_texture.py', 'docs/api/core/renderer/create_external_texture.md'),
        ('core/renderer/create_storage_texture.py', 'docs/api/core/renderer/create_storage_texture.md'),
        ('core/renderer/create_target.py', 'docs/api/core/renderer/create_target.md'),
        ('core/renderer/create_texture.py', 'docs/api/core/renderer/create_texture.md'),
        ('core/renderer/create_texture_target.py', 'docs/api/core/renderer/create_texture_target.md'),
        ('core/renderer/load.py', 'docs/api/core/renderer/load.md'),
        ('core/renderer/new.py', 'docs/api/core/renderer/new.md'),
        ('core/renderer/read_storage.py', 'docs/api/core/renderer/read_storage.md'),
        ('core/renderer/read_texture.py', 'docs/api/core/renderer/read_texture.md'),
        ('core/renderer/render.py', 'docs/api/core/renderer/render.md'),
        ('core/renderer/unregister_texture.py', 'docs/api/core/renderer/unregister_texture.md'),
        ('core/shader/Shader.py', 'docs/api/core/shader/shader.md'),
        ('core/shader/add_mesh.py', 'docs/api/core/shader/add_mesh.md'),
        ('core/shader/clear_meshes.py', 'docs/api/core/shader/clear_meshes.md'),
        ('core/shader/default.py', 'docs/api/core/shader/default.md'),
        ('core/shader/fetch.py', 'docs/api/core/shader/fetch.md'),
        ('core/shader/from_mesh.py', 'docs/api/core/shader/from_mesh.md'),
        ('core/shader/from_vertex.py', 'docs/api/core/shader/from_vertex.md'),
        ('core/shader/get.py', 'docs/api/core/shader/get.md'),
        ('core/shader/is_compute.py', 'docs/api/core/shader/is_compute.md'),
        ('core/shader/list_keys.py', 'docs/api/core/shader/list_keys.md'),
        ('core/shader/list_uniforms.py', 'docs/api/core/shader/list_uniforms.md'),
        ('core/shader/new.py', 'docs/api/core/shader/new.md'),
        ('core/shader/remove_mesh.py', 'docs/api/core/shader/remove_mesh.md'),
        ('core/shader/remove_meshes.py', 'docs/api/core/shader/remove_meshes.md'),
        ('core/shader/set.py', 'docs/api/core/shader/set.md'),
        ('core/shader/set_registry.py', 'docs/api/core/shader/set_registry.md'),
        ('core/shader/validate_mesh.py', 'docs/api/core/shader/validate_mesh.md'),
        ('geometry/instance/Instance.py', 'docs/api/geometry/instance/instance.md'),
        ('geometry/instance/new.py', 'docs/api/geometry/instance/new.md'),
        ('geometry/instance/set.py', 'docs/api/geometry/instance/set.md'),
        ('geometry/mesh/Mesh.py', 'docs/api/geometry/mesh/mesh.md'),
        ('geometry/mesh/add_instance.py', 'docs/api/geometry/mesh/add_instance.md'),
        ('geometry/mesh/add_instances.py', 'docs/api/geometry/mesh/add_instances.md'),
        ('geometry/mesh/add_vertex.py', 'docs/api/geometry/mesh/add_vertex.md'),
        ('geometry/mesh/add_vertices.py', 'docs/api/geometry/mesh/add_vertices.md'),
        ('geometry/mesh/clear_indices.py', 'docs/api/geometry/mesh/clear_indices.md'),
        ('geometry/mesh/clear_instances.py', 'docs/api/geometry/mesh/clear_instances.md'),
        ('geometry/mesh/from_vertices.py', 'docs/api/geometry/mesh/from_vertices.md'),
        ('geometry/mesh/new.py', 'docs/api/geometry/mesh/new.md'),
        ('geometry/mesh/set_indices.py', 'docs/api/geometry/mesh/set_indices.md'),
        ('geometry/mesh/set_instance_count.py', 'docs/api/geometry/mesh/set_instance_count.md'),
        ('geometry/quad/Quad.py', 'docs/api/geometry/quad/quad.md'),
        ('geometry/quad/get_mesh.py', 'docs/api/geometry/quad/get_mesh.md'),
        ('geometry/quad/new.py', 'docs/api/geometry/quad/new.md'),
        ('geometry/vertex/Vertex.py', 'docs/api/geometry/vertex/vertex.md'),
        ('geometry/vertex/create_instance.py', 'docs/api/geometry/vertex/create_instance.md'),
        ('geometry/vertex/new.py', 'docs/api/geometry/vertex/new.md'),
        ('geometry/vertex/pbr.py', 'docs/api/geometry/vertex/pbr.md'),
        ('geometry/vertex/set.py', 'docs/api/geometry/vertex/set.md'),
        ('geometry/vertex_value/VertexValue.py', 'docs/api/geometry/vertex_value/vertex_value.md'),
        ('scene/camera/look_at.py', 'docs/api/scene/camera/look_at.md'),
        ('scene/camera/orthographic.py', 'docs/api/scene/camera/orthographic.md'),
        ('scene/camera/perspective.py', 'docs/api/scene/camera/perspective.md'),
        ('scene/camera/position.py', 'docs/api/scene/camera/position.md'),
        ('scene/camera/set_aspect.py', 'docs/api/scene/camera/set_aspect.md'),
        ('scene/camera/view_proj.py', 'docs/api/scene/camera/view_proj.md'),
        ('scene/light/color.py', 'docs/api/scene/light/color.md'),
        ('scene/light/direction.py', 'docs/api/scene/light/direction.md'),
        ('scene/light/directional.py', 'docs/api/scene/light/directional.md'),
        ('scene/light/inner_cone_angle.py', 'docs/api/scene/light/inner_cone_angle.md'),
        ('scene/light/intensity.py', 'docs/api/scene/light/intensity.md'),
        ('scene/light/kind.py', 'docs/api/scene/light/kind.md'),
        ('scene/light/outer_cone_angle.py', 'docs/api/scene/light/outer_cone_angle.md'),
        ('scene/light/point.py', 'docs/api/scene/light/point.md'),
        ('scene/light/position.py', 'docs/api/scene/light/position.md'),
        ('scene/light/range.py', 'docs/api/scene/light/range.md'),
        ('scene/light/set_color.py', 'docs/api/scene/light/set_color.md'),
        ('scene/light/set_cone_angles.py', 'docs/api/scene/light/set_cone_angles.md'),
        ('scene/light/set_direction.py', 'docs/api/scene/light/set_direction.md'),
        ('scene/light/set_intensity.py', 'docs/api/scene/light/set_intensity.md'),
        ('scene/light/set_position.py', 'docs/api/scene/light/set_position.md'),
        ('scene/light/set_range.py', 'docs/api/scene/light/set_range.md'),
        ('scene/light/spot.py', 'docs/api/scene/light/spot.md'),
        ('scene/material/Material.py', 'docs/api/scene/material/material.md'),
        ('scene/material/alpha_cutoff.py', 'docs/api/scene/material/alpha_cutoff.md'),
        ('scene/material/alpha_mode.py', 'docs/api/scene/material/alpha_mode.md'),
        ('scene/material/base_color.py', 'docs/api/scene/material/base_color.md'),
        ('scene/material/base_color_texture.py', 'docs/api/scene/material/base_color_texture.md'),
        ('scene/material/custom.py', 'docs/api/scene/material/custom.md'),
        ('scene/material/double_sided.py', 'docs/api/scene/material/double_sided.md'),
        ('scene/material/emissive.py', 'docs/api/scene/material/emissive.md'),
        ('scene/material/emissive_texture.py', 'docs/api/scene/material/emissive_texture.md'),
        ('scene/material/metallic.py', 'docs/api/scene/material/metallic.md'),
        ('scene/material/metallic_roughness_texture.py', 'docs/api/scene/material/metallic_roughness_texture.md'),
        ('scene/material/normal_scale.py', 'docs/api/scene/material/normal_scale.md'),
        ('scene/material/normal_texture.py', 'docs/api/scene/material/normal_texture.md'),
        ('scene/material/occlusion_strength.py', 'docs/api/scene/material/occlusion_strength.md'),
        ('scene/material/occlusion_texture.py', 'docs/api/scene/material/occlusion_texture.md'),
        ('scene/material/pbr.py', 'docs/api/scene/material/pbr.md'),
        ('scene/material/roughness.py', 'docs/api/scene/material/roughness.md'),
        ('scene/material/shader.py', 'docs/api/scene/material/shader.md'),
        ('scene/material/uv_transform.py', 'docs/api/scene/material/uv_transform.md'),
        ('scene/model/material.py', 'docs/api/scene/model/material.md'),
        ('scene/model/mesh.py', 'docs/api/scene/model/mesh.md'),
        ('scene/model/new.py', 'docs/api/scene/model/new.md'),
        ('scene/model/rotate.py', 'docs/api/scene/model/rotate.md'),
        ('scene/model/scale.py', 'docs/api/scene/model/scale.md'),
        ('scene/model/set_transform.py', 'docs/api/scene/model/set_transform.md'),
        ('scene/model/set_visible.py', 'docs/api/scene/model/set_visible.md'),
        ('scene/model/transform.py', 'docs/api/scene/model/transform.md'),
        ('scene/model/translate.py', 'docs/api/scene/model/translate.md'),
        ('scene/model/visible.py', 'docs/api/scene/model/visible.md'),
        ('scene/scene/add.py', 'docs/api/scene/scene/add.md'),
        ('scene/scene/add_pass.py', 'docs/api/scene/scene/add_pass.md'),
        ('scene/scene/add_to.py', 'docs/api/scene/scene/add_to.md'),
        ('scene/scene/ambient.py', 'docs/api/scene/scene/ambient.md'),
        ('scene/scene/cameras.py', 'docs/api/scene/scene/cameras.md'),
        ('scene/scene/find_pass.py', 'docs/api/scene/scene/find_pass.md'),
        ('scene/scene/get_pass.py', 'docs/api/scene/scene/get_pass.md'),
        ('scene/scene/lights.py', 'docs/api/scene/scene/lights.md'),
        ('scene/scene/list_passes.py', 'docs/api/scene/scene/list_passes.md'),
        ('scene/scene/load.py', 'docs/api/scene/scene/load.md'),
        ('scene/scene/models.py', 'docs/api/scene/scene/models.md'),
        ('scene/scene/new.py', 'docs/api/scene/scene/new.md'),
        ('scene/scene/no_default_camera.py', 'docs/api/scene/scene/no_default_camera.md'),
        ('scene/scene/no_default_light.py', 'docs/api/scene/scene/no_default_light.md'),
        ('scene/scene/no_defaults.py', 'docs/api/scene/scene/no_defaults.md'),
        ('scene/scene/remove_pass.py', 'docs/api/scene/scene/remove_pass.md'),
        ('scene/scene/set_default_camera.py', 'docs/api/scene/scene/set_default_camera.md'),
        ('scene/scene/set_default_light.py', 'docs/api/scene/scene/set_default_light.md'),
        ('scene/scene/set_passes.py', 'docs/api/scene/scene/set_passes.md'),
        ('targets/target/Target.py', 'docs/api/targets/target/target.md'),
        ('targets/target/get_image.py', 'docs/api/targets/target/get_image.md'),
        ('targets/target/resize.py', 'docs/api/targets/target/resize.md'),
        ('targets/target/size.py', 'docs/api/targets/target/size.md'),
        ('targets/texture_target/TextureTarget.py', 'docs/api/targets/texture_target/texture_target.md'),
        ('targets/texture_target/get_image.py', 'docs/api/targets/texture_target/get_image.md'),
        ('targets/texture_target/resize.py', 'docs/api/targets/texture_target/resize.md'),
        ('targets/texture_target/size.py', 'docs/api/targets/texture_target/size.md'),
        ('targets/texture_target/texture.py', 'docs/api/targets/texture_target/texture.md'),
        ('targets/window_target/WindowTarget.py', 'docs/api/targets/window_target/window_target.md'),
        ('targets/window_target/get_image.py', 'docs/api/targets/window_target/get_image.md'),
        ('targets/window_target/resize.py', 'docs/api/targets/window_target/resize.md'),
        ('targets/window_target/size.py', 'docs/api/targets/window_target/size.md'),
        ('texture/mipmap/Mipmap.py', 'docs/api/texture/mipmap/mipmap.md'),
        ('texture/mipmap/build.py', 'docs/api/texture/mipmap/build.md'),
        ('texture/mipmap/count.py', 'docs/api/texture/mipmap/count.md'),
        ('texture/mipmap/format.py', 'docs/api/texture/mipmap/format.md'),
        ('texture/mipmap/level.py', 'docs/api/texture/mipmap/level.md'),
        ('texture/mipmap/levels.py', 'docs/api/texture/mipmap/levels.md'),
        ('texture/mipmap/size.py', 'docs/api/texture/mipmap/size.md'),
        ('texture/texture/Texture.py', 'docs/api/texture/texture/texture.md'),
        ('texture/texture/aspect.py', 'docs/api/texture/texture/aspect.md'),
        ('texture/texture/get_image.py', 'docs/api/texture/texture/get_image.md'),
        ('texture/texture/id.py', 'docs/api/texture/texture/id.md'),
        ('texture/texture/set_sampler_options.py', 'docs/api/texture/texture/set_sampler_options.md'),
        ('texture/texture/size.py', 'docs/api/texture/texture/size.md'),
        ('texture/texture/write.py', 'docs/api/texture/texture/write.md'),
        ('texture/texture/write_region.py', 'docs/api/texture/texture/write_region.md'),
    ]

    # Announce test count and optionally prepare summary file
    total = len(examples)
    print(f"running {total} tests")
    summary_path = os.environ.get('FC_PY_SUMMARY_PATH')
    if summary_path:
        try:
            with open(summary_path, 'w') as f:
                f.write(f"total={total}\npassed=0\nfailed=0\n")
        except Exception:
            pass

    passed = 0
    failed = 0
    take_gpu_errors()
    for rel, page in examples:
        name = 'platforms.python.examples.' + rel.replace('/', '.').removesuffix('.py')
        head = f'test {name} ... '
        os.environ['FC_RUNNER'] = 'python'
        os.environ['FC_CURRENT_TEST'] = name
        problems = []
        try:
            runpy.run_path(str(base / rel), run_name='__main__')
        except SystemExit as stop:
            if stop.code != 0:
                problems.append(traceback.format_exc())
        except Exception:
            problems.append(traceback.format_exc())
        errors = take_gpu_errors()
        if errors:
            problems.append(f'GPU error: {errors}')
        if problems:
            failed += 1
            print(head + RED + 'FAILED' + RESET)
            print(f'{page}: ' + '\n'.join(problems))
        else:
            passed += 1
            print(head + GREEN + 'OK' + RESET)

    if summary_path:
        try:
            with open(summary_path, 'w') as f:
                f.write(f"total={total}\npassed={passed}\nfailed={failed}\n")
        except Exception:
            pass

    if failed:
        print(f"\n{RED}test result: FAILED{RESET}. {passed} passed; {failed} failed")
        raise SystemExit(1)
    else:
        print(f"\n{GREEN}test result: ok{RESET}. {passed} passed; {failed} failed")

if __name__ == '__main__':
    run_all()
