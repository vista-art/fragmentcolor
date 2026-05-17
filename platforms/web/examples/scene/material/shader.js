import { Material } from "fragmentcolor";

// Direct uniform access for a custom field that isn't covered by the
// Material setters or by Camera / Light.
const material = Material.pbr();
material.shader().set("material.alphaCutoff", 0.25);