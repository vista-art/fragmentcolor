# Auto-generated: executes all Python examples with cargo-like output.
import runpy, pathlib, sys, traceback, os

GREEN='[1;32m'
RED='[1;31m'
RESET='[0m'

def run_all():
    base = pathlib.Path(__file__).parent
    files = [
        'core/frame/Frame.py',
        'core/frame/add_pass.py',
        'core/frame/new.py',
        'core/pass/Pass.py',
        'core/pass/add_depth_target.py',
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
        'core/pass/require.py',
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
        'core/texture/Texture.py',
        'core/texture/aspect.py',
        'core/texture/set_sampler_options.py',
        'core/texture/size.py',
        'geometry/mesh/Mesh.py',
        'geometry/mesh/add_instance.py',
        'geometry/mesh/add_instances.py',
        'geometry/mesh/add_vertex.py',
        'geometry/mesh/add_vertices.py',
        'geometry/mesh/clear_instance_count.py',
        'geometry/mesh/clear_instances.py',
        'geometry/mesh/from_vertices.py',
        'geometry/mesh/new.py',
        'geometry/mesh/set_instance_count.py',
        'geometry/quad/Quad.py',
        'geometry/quad/get_mesh.py',
        'geometry/quad/new.py',
        'geometry/vertex/Vertex.py',
        'geometry/vertex/create_instance.py',
        'geometry/vertex/new.py',
        'geometry/vertex/set.py',
        'targets/target/Target.py',
        'targets/target/get_image.py',
        'targets/target/resize.py',
        'targets/target/size.py',
        'targets/texture_target/TextureTarget.py',
        'targets/texture_target/get_image.py',
        'targets/texture_target/resize.py',
        'targets/texture_target/size.py',
        'targets/window_target/WindowTarget.py',
        'targets/window_target/get_image.py',
        'targets/window_target/resize.py',
        'targets/window_target/size.py',
    ]

    # Announce test count and optionally prepare summary file
    total = len(files)
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
    for rel in files:
        name = 'platforms.python.examples.' + rel.replace('/', '.').removesuffix('.py')
        head = f'test {name} ... '
        os.environ['FC_RUNNER'] = 'python'
        os.environ['FC_CURRENT_TEST'] = name
        try:
            runpy.run_path(str(base / rel), run_name='__main__')
            passed += 1
            print(head + GREEN + 'OK' + RESET)
        except Exception:
            failed += 1
            print(head + RED + 'FAILED' + RESET)
            traceback.print_exc()

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
