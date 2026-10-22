from fragmentcolor import Pass

rpass = Pass("overlay")
rpass.load_previous()

input = rpass.get_input()
keeps = input.loads_previous()