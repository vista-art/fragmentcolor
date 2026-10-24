"""Runs one step of the tutorial in a window: `python host.py 01-three-lines`.

It loads the step's example.py, calls its `setup` once, and calls its `frame`
on every redraw with the seconds since the window opened. With FC_TUTORIAL_FRAMES=n
set, it closes the window after n frames, so a reviewer can run every step in turn.
It needs `pip install fragmentcolor rendercanvas glfw`.
"""

import importlib.util
import inspect
import os
import sys
from pathlib import Path
from time import perf_counter

from fragmentcolor import Renderer
from rendercanvas.auto import RenderCanvas, loop


def load(step):
    path = Path(__file__).parent / step / "example.py"
    spec = importlib.util.spec_from_file_location(step.replace("-", "_"), path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def run(step):
    example = load(step)
    canvas = RenderCanvas(size=(800, 600), title=step, update_mode="continuous")
    renderer = Renderer()
    target = renderer.create_target(canvas)
    takes_renderer = len(inspect.signature(example.setup).parameters) > 0
    state = example.setup(renderer) if takes_renderer else example.setup()

    limit = int(os.environ.get("FC_TUTORIAL_FRAMES", "0"))
    frames = 0
    start = perf_counter()

    @canvas.add_event_handler("resize")
    def resize(event):
        ratio = event["pixel_ratio"]
        target.resize([int(event["width"] * ratio), int(event["height"] * ratio)])

    @canvas.request_draw
    def draw():
        nonlocal frames
        example.frame(state, renderer, target, perf_counter() - start)
        frames += 1
        if limit and frames >= limit:
            print(f"{frames} frames rendered")
            canvas.close()

    loop.run()


if __name__ == "__main__":
    run(sys.argv[1] if len(sys.argv) > 1 else "01-three-lines")
