import { ToonMaterial } from "fragmentcolor";

// Deep blue shadows, a warm middle tone and full light, one texel each.
const ramp = [ 20, 30, 90, 255, 20, 30, 90, 255, 200, 140, 110, 255, 255, 255, 255, 255, ];
const material = new ToonMaterial();
const map = material.rampMap().texture(ramp, [4, 1]);