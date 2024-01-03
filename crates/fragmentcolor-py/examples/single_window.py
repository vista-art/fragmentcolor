import fragmentcolor as plr

scene = plr.Scene()
window = plr.Window(size=(800, 600), title="Hello Gaze Circle")

gaze = plr.Circle(
    color="#ff000088",
    radius=0.05,
    border=0.01,
)


def init():
    scene.target(window)
    scene.add(gaze)

    window.on("draw", on_draw)
    window.auto_update()


def on_draw():
    gaze.set_position(_position_for_time())
    scene.render()
    pass


def _position_for_time():
    return (0.5, 0.5)


if __name__ == '__main__':
    init()
