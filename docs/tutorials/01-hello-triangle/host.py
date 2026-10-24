"""Runs one step of the tutorial in a window: `python host.py 01-three-lines`.

It loads the step's `example.py`, calls its `setup` once, and calls its `frame`
about 60 times a second with the seconds since the window opened. With
`FC_TUTORIAL_FRAMES=n` set, it closes the window after `n` frames.
It needs `pip install fragmentcolor rendercanvas glfw`.
"""

import importlib.util
import inspect
import os
import sys
from pathlib import Path
from time import perf_counter

from fragmentcolor import FragmentColorError, Renderer
from rendercanvas.auto import RenderCanvas, loop


def load(step):
    path = Path(__file__).parent / step / "example.py"
    spec = importlib.util.spec_from_file_location(step.replace("-", "_"), path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def run(step):
    example = load(step)
    canvas = RenderCanvas(size=(800, 600), title=step)
    renderer = Renderer()
    target = renderer.create_target(canvas)
    takes_renderer = len(inspect.signature(example.setup).parameters) > 0
    state = example.setup(renderer) if takes_renderer else example.setup()

    limit = int(os.environ.get("FC_TUTORIAL_FRAMES", "0"))
    frames = 0
    start = perf_counter()

    # `render` presents each frame itself, so a timer drives the frames.
    def tick():
        nonlocal frames
        if canvas.get_closed():
            return
        width, height = canvas.get_physical_size()
        size = target.size()
        if (size.width, size.height) != (width, height):
            target.resize([width, height])

        try:
            example.frame(state, renderer, target, perf_counter() - start)
        except FragmentColorError as error:
            # A hidden or minimized window has nothing to draw on, so the frame is skipped.
            if "occluded" not in str(error):
                raise
        frames += 1
        if limit and frames >= limit:
            print(f"{frames} frames rendered")
            canvas.close()
            return
        loop.call_later(1 / 60, tick)

    loop.call_soon(tick)
    loop.run()


if __name__ == "__main__":
    run(sys.argv[1] if len(sys.argv) > 1 else "01-three-lines")
