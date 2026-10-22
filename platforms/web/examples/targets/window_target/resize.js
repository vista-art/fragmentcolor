import { Renderer } from "fragmentcolor";
const canvas = document.createElement("canvas");

const renderer = new Renderer();
const target = await renderer.createTarget(canvas);

target.resize([1024, 768]);
