import { Renderer } from "fragmentcolor";
const canvas = document.createElement("canvas");

const renderer = new Renderer();
const target = await renderer.createTarget(canvas);
const size = target.size();
