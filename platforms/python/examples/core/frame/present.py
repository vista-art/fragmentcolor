from fragmentcolor import Frame, Pass

shadow = Pass("shadow")
main = Pass("main")

frame = Frame()
frame.add_pass(shadow)
frame.add_pass(main)
frame.connect(shadow, main)
frame.present(main)