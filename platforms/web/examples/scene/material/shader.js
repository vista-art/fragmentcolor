import { Material, Renderer } from "fragmentcolor";

// Direct uniform access for a custom field that isn't covered by the
// Material setters or by Camera / Light.
const renderer = new Renderer();
const material = Material.pbr();
material.shader().set("material.alphaCutoff", 0.25);