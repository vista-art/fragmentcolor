"""The context FragmentColor registers on a rendercanvas canvas.

rendercanvas calls a draw function only for a canvas that has a context.
`Renderer.render` presents each frame itself, so this context only tells
rendercanvas that the frame is on screen.
"""

from rendercanvas.contexts import BaseContext


class FragmentColorContext(BaseContext):
    present_methods = ["screen"]

    def _rc_present(self, *, force_sync=False):
        return {"method": "screen"}
