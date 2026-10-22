from fragmentcolor import Pass

rpass = Pass("background")
rpass.set_clear_color([1.0, 0.0, 0.0, 1.0])
color = rpass.get_input().color()