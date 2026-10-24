import { Renderer } from "fragmentcolor";
const image = new Uint8Array(await (await fetch("/healthcheck/public/favicon.png")).arrayBuffer());
const renderer = new Renderer();
const tex = await renderer.createTexture(image);