import plrender as plr

window = plr.Window(width=400, heigth=300,
                    title="Spritesheet Example", clear_color="#FFccffff")


def loop():
    pass


if __name__ == '__main__':
    window.on_update = loop
    window.run()
