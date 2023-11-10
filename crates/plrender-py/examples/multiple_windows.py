import plrender as plr


def init():
    window1 = plr.Window(size=(800, 600), title="Hello First Window")
    window2 = plr.Window(size=(400, 300), title="Hello Second Window")

    window1.on("resize", lambda width, height: print(
        "Window 1 resized to", width, height))
    window2.on("resize", lambda width, height: print(
        "Window 2 resized to", width, height))

    plr.run()


if __name__ == '__main__':
    init()
