// Ink lines where the scene's depth or normals break: along a silhouette,
// where a surface ends in front of another, and along a crease, where two
// faces of a surface meet at an angle.
//
// A fullscreen triangle, drawn after the scene with alpha blending, so the
// ink lands on the colors the scene drew. Each pixel compares itself with the
// pixels `width` away from it, in the depth texture and the normals texture
// an earlier pass drew.

struct Outline {
  // The ink's color. Its alpha is the ink's opacity.
  color: vec4<f32>,
  // The camera's near and far planes, which turn a depth back into a distance.
  near: f32,
  far: f32,
  // How many pixels apart the compared pixels are, which sets the line width.
  width: f32,
  // How much farther a neighbor must be, as a fraction of this pixel's
  // distance, for this pixel to sit on a silhouette.
  silhouette: f32,
  // How far a neighbor's normal must turn for a crease, as the distance between
  // the two unit normals: 0.5 is about 29 degrees and 1.0 is 60.
  crease: f32,
}

@group(0) @binding(0) var depth: texture_depth_2d;
@group(0) @binding(1) var normals: texture_2d<f32>;
@group(0) @binding(2) var<uniform> outline: Outline;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
  let corner = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
  return vec4<f32>(corner * 2.0 - 1.0, 0.0, 1.0);
}

// The distance along the view axis from the camera to the surface at `pixel`.
// A pixel no surface covers keeps the cleared depth, 1, and reads as the far plane.
fn distance_at(pixel: vec2<i32>) -> f32 {
  let stored = textureLoad(depth, pixel, 0);
  return outline.near * outline.far / (outline.far - stored * (outline.far - outline.near));
}

// The unit normal at `pixel`, with its coverage in `w`: 1 on a surface, 0 on the background.
fn normal_at(pixel: vec2<i32>) -> vec4<f32> {
  let texel = textureLoad(normals, pixel, 0);
  return vec4<f32>(texel.xyz * 2.0 - 1.0, texel.a);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
  let pixel = vec2<i32>(position.xy);
  let last = vec2<i32>(textureDimensions(depth)) - 1;
  let reach = max(i32(round(outline.width)), 1);
  let here = distance_at(pixel);
  let normal = normal_at(pixel);

  // Right, down, left and up.
  let sides = array<vec2<i32>, 4>(vec2<i32>(1, 0), vec2<i32>(0, 1), vec2<i32>(-1, 0), vec2<i32>(0, -1));
  var ink = 0.0;
  for (var side = 0; side < 4; side++) {
    let there = clamp(pixel + sides[side] * reach, vec2<i32>(0), last);

    // A neighbor much farther away puts this pixel on the near side of a
    // silhouette, so the line stays on the surface in front.
    if (distance_at(there) - here > outline.silhouette * here) {
      ink = 1.0;
    }

    // Between two covered pixels, a normal that turns marks a crease. Only the
    // right and lower neighbors count, so the line is drawn on one side of it.
    let other = normal_at(there);
    let covered = normal.w * other.w > 0.5;
    if (side < 2 && covered && distance(other.xyz, normal.xyz) > outline.crease) {
      ink = 1.0;
    }
  }
  return vec4<f32>(outline.color.rgb, outline.color.a * ink);
}
