
import { Renderer } from "fragmentcolor";
const canvas = document.createElement("canvas");

const renderer = new Renderer();

// The canvas comes from your platform's windowing system.
const target = await renderer.createTarget(canvas);
