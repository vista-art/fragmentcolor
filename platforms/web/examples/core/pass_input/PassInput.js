import { Pass } from "fragmentcolor";

const pass = new Pass("overlay");
pass.loadPrevious();

const input = pass.getInput();
const keeps = input.loadsPrevious();