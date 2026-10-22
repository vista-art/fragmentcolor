import { Pass } from "fragmentcolor";

const pass = new Pass("background");
pass.setClearColor([1.0, 0.0, 0.0, 1.0]);
const color = pass.getInput().color();