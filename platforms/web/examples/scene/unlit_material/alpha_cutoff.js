import { AlphaMode, UnlitMaterial } from "fragmentcolor";

const sprite = new UnlitMaterial().alphaMode(AlphaMode.Mask).alphaCutoff(0.3);