import FragmentColor

let pass = Pass("background")
try pass.setClearColor([1.0, 0.0, 0.0, 1.0])
let color = pass.getInput().color()