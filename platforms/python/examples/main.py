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
        'core/frame/new.py',
        'core/pass/Pass.py',
        'core/pass/add_shader.py',
        'core/pass/from_shader.py',
        'core/pass/new.py',
        'core/pass/set_clear_color.py',
        'core/renderer/Renderer.py',
        'core/renderer/create_target.py',
        'core/renderer/create_texture_target.py',
        'core/renderer/new.py',
        'core/renderer/render.py',
        'core/shader/Shader.py',
        'core/shader/get.py',
        'core/shader/list_keys.py',
        'core/shader/list_uniforms.py',
        'core/shader/new.py',
        'core/shader/set.py',
        'core/target/Target.py',
        'core/target/get_current_frame.py',
        'core/target/get_image.py',
        'core/target/resize.py',
        'core/target/size.py',
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
