import { AlphaMode, UnlitMaterial } from "fragmentcolor";

const overlay = new UnlitMaterial().baseColor([0.0, 0.0, 0.0, 0.5]).alphaMode(AlphaMode.Blend);