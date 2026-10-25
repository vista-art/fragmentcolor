"""Runs one step of the tutorial in a window: `python host.py 02-on-the-gpu`.

It loads the step's `example.py`, calls its `setup` once, and calls its `frame`
about 60 times a second with the seconds since the window opened, the window's
size, and the pointer, all in pixels with y pointing down. With
`FC_TUTORIAL_FRAMES=n` set, it closes the window after `n` frames.
It needs `pip install fragmentcolor rendercanvas glfw`.
"""

import importlib.util
import os
import sys
from pathlib import Path
from time import perf_counter

from fragmentcolor import OccludedSurfaceError, Renderer
from rendercanvas.auto import RenderCanvas, loop


def load(step):
    path = Path(__file__).parent / step / "example.py"
    spec = importlib.util.spec_from_file_location(step.replace("-", "_"), path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def run(step):
    example = load(step)
    canvas = RenderCanvas(size=(960, 600), title=step)
    renderer = Renderer()
    target = renderer.create_target(canvas)
    state = example.setup()

    # The pointer in pixels, y down. Until it moves over the window, frame gets the center.
    pointer = None

    def track(event):
        nonlocal pointer
        ratio = canvas.get_pixel_ratio()
        pointer = [event["x"] * ratio, event["y"] * ratio]

    canvas.add_event_handler(track, "pointer_move")

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
            here = pointer or [width / 2, height / 2]
            example.frame(state, renderer, target, perf_counter() - start, [width, height], here)
        except OccludedSurfaceError:
            # A hidden or minimized window has nothing to draw on, so the frame is skipped.
            pass
        frames += 1
        if limit and frames >= limit:
            print(f"{frames} frames rendered")
            canvas.close()
            return
        loop.call_later(1 / 60, tick)

    loop.call_soon(tick)
    loop.run()


if __name__ == "__main__":
    run(sys.argv[1] if len(sys.argv) > 1 else "02-on-the-gpu")
