# Auto-generated: executes all Python examples.
import runpy, pathlib

def run_all():
    base = pathlib.Path(__file__).parent
    files = [
        'core/renderer/constructor.py',
        'core/renderer/create_texture_target.py',
        'core/renderer/render.py',
        'core/shader/constructor.py',
        'core/shader/get.py',
        'core/shader/list_keys.py',
        'core/shader/list_uniforms.py',
        'core/shader/set.py',
        'core/pass/constructor.py',
        'core/pass/add_shader.py',
        'core/frame/constructor.py',
        'core/frame/add_pass.py',
    ]
    for rel in files:
        runpy.run_path(str(base / rel), run_name='__main__')
