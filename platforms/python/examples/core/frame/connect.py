from fragmentcolor import Frame, Pass

p1 = Pass("shadow")
p2 = Pass("main")

frame = Frame()
f.add_pass(p1)
f.add_pass(p2)

frame.connect(p1, p2)