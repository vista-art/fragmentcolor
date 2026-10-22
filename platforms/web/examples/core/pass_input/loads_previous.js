import { Pass } from "fragmentcolor";

const pass = new Pass("overlay");
pass.loadPrevious();
const keeps = pass.getInput().loadsPrevious();