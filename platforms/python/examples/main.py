# Auto-generated: executes all Python examples with cargo-like output.
import runpy, pathlib, sys, traceback

GREEN='[1;32m'
RED='[1;31m'
RESET='[0m'

def run_all():
    base = pathlib.Path(__file__).parent
    files = [
        'core/frame/Frame.py',
        'core/frame/add_pass.py',
        'core/frame/connect.py',
        'core/frame/new.py',
        'core/frame/present.py',
        'core/mesh/Mesh.py',
        'core/mesh/add_instance.py',
        'core/mesh/add_instances.py',
        'core/mesh/add_vertex.py',
        'core/mesh/add_vertices.py',
        'core/mesh/clear_instance_count.py',
        'core/mesh/clear_instances.py',
        'core/mesh/from_vertices.py',
        'core/mesh/new.py',
        'core/mesh/primitives/quad/Quad.py',
        'core/mesh/primitives/quad/get_mesh.py',
        'core/mesh/primitives/quad/new.py',
        'core/mesh/set_instance_count.py',
        'core/pass/Pass.py',
        'core/pass/add_mesh.py',
        'core/pass/add_mesh_to_shader.py',
        'core/pass/add_shader.py',
        'core/pass/add_target.py',
        'core/pass/compute.py',
        'core/pass/from_shader.py',
        'core/pass/get_input.py',
        'core/pass/is_compute.py',
        'core/pass/load_previous.py',
        'core/pass/new.py',
        'core/pass/set_clear_color.py',
        'core/pass/set_compute_dispatch.py',
        'core/pass/set_viewport.py',
        'core/renderer/Renderer.py',
        'core/renderer/create_depth_texture.py',
        'core/renderer/create_storage_texture.py',
        'core/renderer/create_target.py',
        'core/renderer/create_texture.py',
        'core/renderer/create_texture_target.py',
        'core/renderer/create_texture_with.py',
        'core/renderer/create_texture_with_format.py',
        'core/renderer/create_texture_with_size.py',
        'core/renderer/new.py',
        'core/renderer/render.py',
        'core/shader/Shader.py',
        'core/shader/add_mesh.py',
        'core/shader/clear_meshes.py',
        'core/shader/from_mesh.py',
        'core/shader/from_vertex.py',
        'core/shader/get.py',
        'core/shader/is_compute.py',
        'core/shader/list_keys.py',
        'core/shader/list_uniforms.py',
        'core/shader/new.py',
        'core/shader/remove_mesh.py',
        'core/shader/remove_meshes.py',
        'core/shader/set.py',
        'core/shader/validate_mesh.py',
        'core/target/Target.py',
        'core/target/get_current_frame.py',
        'core/target/get_image.py',
        'core/target/resize.py',
        'core/target/size.py',
        'core/texture/Texture.py',
        'core/texture/aspect.py',
        'core/texture/set_sampler_options.py',
        'core/texture/size.py',
        'core/vertex/Vertex.py',
        'core/vertex/create_instance.py',
        'core/vertex/new.py',
        'core/vertex/set.py',
        'targets/texture_target/TextureTarget.py',
        'targets/texture_target/get_image.py',
        'targets/texture_target/resize.py',
        'targets/texture_target/size.py',
        'targets/window_target/WindowTarget.py',
        'targets/window_target/get_image.py',
        'targets/window_target/resize.py',
        'targets/window_target/size.py',
    ]
    failed = 0
    for rel in files:
        name = 'platforms.python.examples.' + rel.replace('/', '.').removesuffix('.py')
        head = f'test {name} ... '
        try:
            runpy.run_path(str(base / rel), run_name='__main__')
            print(head + GREEN + 'OK' + RESET)
        except Exception:
            failed += 1
            print(head + RED + 'FAILED' + RESET)
            traceback.print_exc()
    if failed:
        raise SystemExit(1)

if __name__ == '__main__':
    run_all()
