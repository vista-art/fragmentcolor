import plrender as plr


def init():
    app = plr.App()
    window1 = plr.Window(app, size=(800, 600), title="Hello World")
    window2 = plr.Window(app, size=(400, 300), title="Hello Second World")

    window1.on("resize", lambda width, height: print(
        "Window 1 resized to", width, height))
    window2.on("resize", lambda width, height: print(
        "Window 2 resized to", width, height))

    app.run()


if __name__ == '__main__':
    init()
