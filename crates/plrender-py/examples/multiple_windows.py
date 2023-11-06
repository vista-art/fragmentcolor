import plrender as plr


def init():
    app = plr.App()
    plr.Window(app, size=(800, 600), title="Hello World")
    plr.Window(app, size=(400, 300), title="Hello Second World")
    app.run()


if __name__ == '__main__':
    init()
