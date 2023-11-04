import plrender


def init():
    print("creating App...")
    app = plrender.App()

    print("creating Window...")
    plrender.Window(app, size=(800, 600), title="Hello World")

    print("running App...")
    app.run()


if __name__ == '__main__':
    init()
