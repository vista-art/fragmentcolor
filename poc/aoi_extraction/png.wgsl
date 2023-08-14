@group(0) @binding(0)
var img_input: texture_2d<u32>;
@group(0) @binding(1)
var img_output: texture_storage_3d<rgba8uint,write>;

@compute @workgroup_size(1, 1, 1)
fn main(@builtin(global_invocation_id) param: vec3<u32>) {
    var pos = vec2<i32>(param.xy);
    var color = textureLoad(img_input, pos, 0);

    for (var aoi: u32 = 0u; aoi < 32u; aoi++) {
        var output_color = vec4(vec3(0u), 255u);
        var color_channel = aoi / 8u;
        var channel_bitmask = 1u << (aoi % 8u);

        if (color[color_channel] & channel_bitmask) != 0u {
            output_color = vec4<u32>(255u);
        }

        textureStore(img_output, vec3<i32>(pos.x, pos.y, i32(aoi)), output_color);
    }
}
