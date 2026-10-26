"""Runs one step of the tutorial in a window: `python host.py 02-on-the-gpu`.

It loads the step's `example.py`, calls its `setup` once, and calls its `frame`
about 60 times a second with the seconds since the window opened, the window's
size, and the pointer, all in pixels with y pointing down. A `frame` that takes
a seventh argument also gets the wheel and left-button drags since the last
frame, as `{"zoom": doublings, "pan": [dx, dy]}`, where a positive zoom goes in.
With `FC_TUTORIAL_FRAMES=n` set, it closes the window after `n` frames.
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
    canvas = RenderCanvas(size=(960, 600), title=step)
    renderer = Renderer()
    target = renderer.create_target(canvas)
    state = example.setup()

    # The pointer in pixels, y down. Until it moves over the window, frame gets the center.
    pointer = None
    # The wheel and left-button drags since the last frame, for a frame that takes them.
    gesture = {"zoom": 0.0, "pan": [0.0, 0.0]}
    gestures = len(inspect.signature(example.frame).parameters) > 6

    def track(event):
        nonlocal pointer
        ratio = canvas.get_pixel_ratio()
        here = [event["x"] * ratio, event["y"] * ratio]
        # A drag with the left button held pans as far as the pointer moved.
        if pointer and 1 in event["buttons"]:
            gesture["pan"][0] += here[0] - pointer[0]
            gesture["pan"][1] += here[1] - pointer[1]
        pointer = here

    # rendercanvas reports a wheel notch as 100, below zero when the wheel turns away
    # from the reader, so each notch away zooms in a quarter doubling, as on the web.
    def scroll(event):
        gesture["zoom"] -= event["dy"] / 400

    canvas.add_event_handler(track, "pointer_move")
    canvas.add_event_handler(scroll, "wheel")

    limit = int(os.environ.get("FC_TUTORIAL_FRAMES", "0"))
    frames = 0
    start = perf_counter()

    # `render` presents each frame itself, so a timer drives the frames.
    def tick():
        nonlocal frames, gesture
        if canvas.get_closed():
            return
        width, height = canvas.get_physical_size()
        size = target.size()
        if (size.width, size.height) != (width, height):
            target.resize([width, height])

        here = pointer or [width / 2, height / 2]
        arguments = [state, renderer, target, perf_counter() - start, [width, height], here]
        if gestures:
            arguments.append(gesture)
            gesture = {"zoom": 0.0, "pan": [0.0, 0.0]}
        try:
            example.frame(*arguments)
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
    run(sys.argv[1] if len(sys.argv) > 1 else "02-on-the-gpu")
