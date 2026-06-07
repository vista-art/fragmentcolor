import { Pass, Renderer, Shader } from "fragmentcolor";

const renderer = new Renderer();
const target = await renderer.createTextureTarget([16, 16]);

const compute = new Shader(`

    struct Out { values: array<f32, 4> };
    @group(0) @binding(0) var<storage, read_write> out: Out;
    @compute @workgroup_size(1) fn main() {
        out.values[0] = 1.0;
        out.values[1] = 2.0;
        out.values[2] = 3.0;
        out.values[3] = 4.0;
    }
    
`);

const pass = Pass.compute("seed");
pass.setComputeDispatch(1, 1, 1);
pass.addShader(compute);
renderer.render(pass, target);

const bytes = await renderer.readStorage(compute, "out");