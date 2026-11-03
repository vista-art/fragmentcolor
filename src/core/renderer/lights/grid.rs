//! The light grid on the CPU: its depth slices from the binning matrix, the
//! bounding sphere of each light, and the box of clusters each sphere covers.
//! `light_cell` mirrors the WGSL function of the same name.

use glam::{Mat4, Vec3, Vec4, Vec4Swizzles};

use crate::scene::GpuLight;

/// Tiles along the long side of the grid's view.
pub(crate) const GRID_LONG: u32 = 16;
/// Tiles along the short side of the grid's view.
pub(crate) const GRID_SHORT: u32 = 9;
/// Depth slices.
pub(crate) const GRID_SLICES: u32 = 24;
/// How far outside the grid, in clusters, a point still takes the nearest cluster.
pub(crate) const GRID_MARGIN: f32 = 1.0 / 64.0;
/// Clusters in a grid, either way up.
pub(crate) const GRID_CELLS: u32 = GRID_LONG * GRID_SHORT * GRID_SLICES;
/// Bits per bound in a packed cluster box.
const BOX_BITS: u32 = 5;
/// How far a light's box reaches past its sphere: the grid margin, and as
/// much again for rounding between the CPU and the GPU.
const BOX_MARGIN: f32 = 2.0 * GRID_MARGIN;
/// How much the GPU widens a sphere before testing it, which reach matches.
const SPHERE_SLACK: f32 = 1.0e-4;

/// How depth maps to slices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Slicing {
    /// Slices grow with distance, for a perspective view.
    Log,
    /// Slices of equal depth, for an orthographic view.
    Linear,
}

impl Slicing {
    /// The value `LightGrid.slicing` holds.
    pub(crate) fn encode(self) -> u32 {
        match self {
            Slicing::Log => 1,
            Slicing::Linear => 2,
        }
    }
}

/// A sphere around everything a light reaches.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Sphere {
    pub(crate) center: Vec3,
    pub(crate) radius: f32,
}

impl Sphere {
    /// The bounding sphere of `light`, or `None` for a light that reaches everywhere:
    /// a directional light, and a light without a finite range or position.
    pub(crate) fn around(light: &GpuLight) -> Option<Self> {
        let position = Vec3::from(light.position);
        let range = light.range;
        if light.kind == 0 || !position.is_finite() || !range.is_finite() || range <= 0.0 {
            return None;
        }
        let full = Sphere {
            center: position,
            radius: range,
        };
        if light.kind != 2 {
            return Some(full);
        }
        let axis = Vec3::from(light.direction);
        let length = axis.length();
        if !axis.is_finite() || !length.is_finite() || length <= 0.0 {
            return Some(full);
        }
        if !light.outer_cone_cos.is_finite() {
            return Some(full);
        }
        let axis = axis / length;
        let angle =
            (light.outer_cone_cos.clamp(-1.0, 1.0).acos() + 0.01).min(std::f32::consts::FRAC_PI_2);
        if angle <= std::f32::consts::FRAC_PI_4 {
            let radius = range / (2.0 * angle.cos());
            Some(Sphere {
                center: position + axis * radius,
                radius,
            })
        } else {
            Some(Sphere {
                center: position + axis * range * angle.cos(),
                radius: range * angle.sin(),
            })
        }
    }
}

/// The view a grid bins in, before the lights set where its slices end.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Frustum {
    /// The binning matrix, scaled so the depth plane has a unit normal.
    pub(crate) view_proj: Mat4,
    pub(crate) depth_plane: Vec4,
    pub(crate) slicing: Slicing,
    /// The depths of the near and far planes, nearest first.
    pub(crate) near: f32,
    pub(crate) far: f32,
    pub(crate) size: [u32; 3],
}

/// How close to 0 or 1 the depth plane's share of clip depth may come before
/// its plane lies past what the matrix's f32 entries resolve.
const PLANE_RESOLUTION: f64 = 1.0e-6;

/// The view depths of a perspective `matrix`'s near and far planes, nearest
/// first, worked in f64. A plane past what the f32 entries resolve, as a far
/// of a million times the near gives, comes back infinite. `None` when a plane
/// lies behind the eye or neither plane resolves.
fn find_planes(matrix: &Mat4) -> Option<(f32, f32)> {
    let plane = matrix.row(3).as_dvec4();
    let normal = plane.xyz().length();
    let (plane, depth) = (plane / normal, matrix.row(2).as_dvec4() / normal);
    // Clip depth over view depth is `a + b / depth`, which is 0 at one plane and 1 at the other.
    let a = depth.xyz().dot(plane.xyz());
    let b = depth.w - a * plane.w;
    let solve = |share: f64, value: f64| match share.abs() < PLANE_RESOLUTION {
        true => f64::INFINITY,
        false => value / share,
    };
    let (first, second) = (solve(a, -b), solve(1.0 - a, b));
    let near = first.min(second);
    if !near.is_finite() || near <= 0.0 || first.is_nan() || second.is_nan() {
        return None;
    }
    Some((near as f32, first.max(second) as f32))
}

impl Frustum {
    /// The frustum of `matrix`, or `None` when it can't bin: a matrix that
    /// isn't finite, or a perspective whose near plane isn't in front of the eye.
    pub(crate) fn from_matrix(matrix: Mat4) -> Option<Self> {
        if !matrix.is_finite() {
            return None;
        }
        let row = matrix.row(3);
        let normal = row.xyz().length();
        let (view_proj, depth_plane, slicing, near, far) = if normal > 1.0e-6 * row.w.abs().max(1.0)
        {
            let view_proj = matrix * (1.0 / normal);
            let (near, far) = find_planes(&matrix)?;
            (view_proj, view_proj.row(3), Slicing::Log, near, far)
        } else {
            if row.w.is_nan() || row.w <= 0.0 {
                return None;
            }
            let view_proj = matrix * (1.0 / row.w);
            let depth = view_proj.row(2);
            let normal = depth.xyz().length();
            if normal.is_nan() || normal <= 0.0 {
                return None;
            }
            (
                view_proj,
                depth / normal,
                Slicing::Linear,
                0.0,
                1.0 / normal,
            )
        };
        if !view_proj.is_finite() || !depth_plane.is_finite() || far.is_nan() {
            return None;
        }
        let portrait = view_proj.row(0).xyz().length() > view_proj.row(1).xyz().length();
        let size = match portrait {
            true => [GRID_SHORT, GRID_LONG, GRID_SLICES],
            false => [GRID_LONG, GRID_SHORT, GRID_SLICES],
        };
        Some(Frustum {
            view_proj,
            depth_plane,
            slicing,
            near,
            far,
            size,
        })
    }

    /// The depth of `point` along the depth plane.
    pub(crate) fn measure_depth(&self, point: Vec3) -> f32 {
        self.depth_plane.dot(point.extend(1.0))
    }

    /// The deepest point any of `spheres` reaches, widened as the GPU widens them.
    pub(crate) fn reach(&self, spheres: &[Sphere]) -> f32 {
        spheres
            .iter()
            .map(|sphere| {
                let depth =
                    self.measure_depth(sphere.center) + sphere.radius * (1.0 + SPHERE_SLACK);
                depth + depth.abs() * SPHERE_SLACK
            })
            .fold(f32::NEG_INFINITY, f32::max)
    }

    /// The grid whose slices end at `reach`, or at the far plane before it.
    pub(crate) fn build_grid(&self, reach: f32) -> Grid {
        let slices = GRID_SLICES as f32;
        let (scale, bias) = match self.slicing {
            Slicing::Log => {
                let end = reach.min(self.far).max(1.01 * self.near);
                let scale = slices / (end / self.near).log2();
                (scale, -self.near.log2() * scale)
            }
            Slicing::Linear => {
                let end = reach
                    .min(self.far)
                    .max(self.near + 1.0e-3 * (self.far - self.near));
                let scale = slices / (end - self.near);
                (scale, -self.near * scale)
            }
        };
        Grid {
            frustum: *self,
            scale,
            bias,
            reach,
        }
    }
}

/// The clusters a light covers, inclusive on both ends of each axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Cells {
    pub(crate) x: [u32; 2],
    pub(crate) y: [u32; 2],
    pub(crate) z: [u32; 2],
}

impl Cells {
    /// The number of clusters in the box.
    pub(crate) fn count_cells(&self) -> u32 {
        let span = |[low, high]: [u32; 2]| high - low + 1;
        span(self.x) * span(self.y) * span(self.z)
    }

    /// The box packed as `Bin.cells` holds it, five bits per bound.
    pub(crate) fn pack(&self) -> u32 {
        [
            self.x[0], self.x[1], self.y[0], self.y[1], self.z[0], self.z[1],
        ]
        .iter()
        .enumerate()
        .fold(0, |packed, (index, bound)| {
            packed | (bound << (BOX_BITS * index as u32))
        })
    }
}

/// A frustum cut into clusters, with slices that end where the lights do.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Grid {
    pub(crate) frustum: Frustum,
    pub(crate) scale: f32,
    pub(crate) bias: f32,
    pub(crate) reach: f32,
}

impl Grid {
    /// The continuous slice coordinate of `depth`.
    pub(crate) fn find_slice(&self, depth: f32) -> f32 {
        match self.frustum.slicing {
            Slicing::Log if depth <= 0.0 => f32::NEG_INFINITY,
            Slicing::Log => depth.log2() * self.scale + self.bias,
            Slicing::Linear => depth * self.scale + self.bias,
        }
    }

    /// The clusters `sphere` may reach, widened by the margin, or `None` when
    /// it reaches none of them.
    pub(crate) fn find_cells(&self, sphere: &Sphere) -> Option<Cells> {
        let [width, height, slices] = self.frustum.size;
        let matrix = &self.frustum.view_proj;
        let point = sphere.center.extend(1.0);
        let radius = sphere.radius;
        let w = matrix.row(3).dot(point);
        let slope = matrix.row(3).xyz().length();
        let room = w - radius * slope;
        let (x, y) = if room > 0.0 {
            let axis = |row: Vec4, size: u32| {
                let center = row.dot(point) / w;
                let half = radius * (row.xyz().length() + center.abs() * slope) / room;
                let to_grid = |ndc: f32| (ndc * 0.5 + 0.5) * size as f32;
                find_span(to_grid(center - half), to_grid(center + half), size)
            };
            (axis(matrix.row(0), width)?, axis(matrix.row(1), height)?)
        } else {
            ([0, width - 1], [0, height - 1])
        };
        let depth = self.frustum.measure_depth(sphere.center);
        let z = find_span(
            self.find_slice(depth - radius),
            self.find_slice(depth + radius),
            slices,
        )?;
        Some(Cells { x, y, z })
    }

    /// The `LightGrid` fields from `depth_plane` on: size, slicing, scale, bias and reach.
    pub(crate) fn write_tail(&self, out: &mut Vec<u8>) {
        for value in self.frustum.depth_plane.to_array() {
            out.extend_from_slice(&value.to_le_bytes());
        }
        for side in self.frustum.size {
            out.extend_from_slice(&side.to_le_bytes());
        }
        out.extend_from_slice(&self.frustum.slicing.encode().to_le_bytes());
        for value in [self.scale, self.bias] {
            out.extend_from_slice(&value.to_le_bytes());
        }
    }
}

/// The cluster range from continuous coordinates `low` to `high` on an axis
/// of `size` clusters, widened by the box margin, or `None` when it misses the axis.
/// A coordinate that isn't a number takes the whole axis.
fn find_span(low: f32, high: f32, size: u32) -> Option<[u32; 2]> {
    let last = (size - 1) as f32;
    let low = (low - BOX_MARGIN).floor();
    let high = (high + BOX_MARGIN).floor();
    if high < 0.0 || low > last {
        return None;
    }
    Some([
        low.max(0.0).min(last) as u32,
        high.min(last).max(0.0) as u32,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::camera::rh::{proj::directx, view};

    impl Cells {
        /// Whether the box holds the cluster at `cell`.
        fn holds(&self, cell: [u32; 3]) -> bool {
            let inside = |[low, high]: [u32; 2], value: u32| low <= value && value <= high;
            inside(self.x, cell[0]) && inside(self.y, cell[1]) && inside(self.z, cell[2])
        }
    }

    /// Where a point reads its lights from, as `light_cell` decides.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Cell {
        /// Every light: the point is behind the grid's eye or outside the grid.
        Every,
        /// Only the lights without a range: no other light reaches this deep.
        Global,
        /// The cluster at these coordinates.
        Cluster([u32; 3]),
    }

    impl Grid {
        /// The depth where slice coordinate `slice` lies, as the binning shader's `depth_at`.
        pub(crate) fn depth_at(&self, slice: f32) -> f32 {
            let value = (slice - self.bias) / self.scale;
            match self.frustum.slicing {
                Slicing::Log => value.exp2(),
                Slicing::Linear => value,
            }
        }

        /// Where a fragment at `world` reads its lights from, as the WGSL `light_cell` decides.
        fn light_cell(&self, world: Vec3) -> Cell {
            let point = world.extend(1.0);
            let depth = self.frustum.depth_plane.dot(point);
            if depth > self.reach {
                return Cell::Global;
            }
            let clip = self.frustum.view_proj * point;
            if clip.w <= 0.0 {
                return Cell::Every;
            }
            let size = self.frustum.size.map(|side| side as f32);
            let slice = match self.frustum.slicing {
                Slicing::Log => depth.max(1.0e-30).log2() * self.scale + self.bias,
                Slicing::Linear => depth * self.scale + self.bias,
            };
            let cell = [
                (clip.x / clip.w * 0.5 + 0.5) * size[0],
                (clip.y / clip.w * 0.5 + 0.5) * size[1],
                slice,
            ];
            let outside = cell
                .iter()
                .zip(size)
                .any(|(value, side)| *value < -GRID_MARGIN || *value > side + GRID_MARGIN);
            if outside {
                return Cell::Every;
            }
            let index = |axis: usize| (cell[axis].max(0.0) as u32).min(self.frustum.size[axis] - 1);
            Cell::Cluster([index(0), index(1), index(2)])
        }
    }

    fn camera(projection: Mat4) -> Mat4 {
        projection * view::look_at_mat4(Vec3::new(0.0, 1.0, 6.0), Vec3::ZERO, Vec3::Y)
    }

    fn perspective(near: f32, far: f32) -> Mat4 {
        camera(directx::perspective(1.0, 1.5, near, far))
    }

    /// A small deterministic generator, so the samples repeat run to run.
    struct Seeds(u64);

    impl Seeds {
        fn next(&mut self) -> f32 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((self.0 >> 40) as f32) / (1u64 << 24) as f32
        }

        fn signed(&mut self) -> f32 {
            self.next() * 2.0 - 1.0
        }
    }

    #[test]
    fn depth_planes_come_from_the_matrix() {
        let forward = Frustum::from_matrix(perspective(0.1, 100.0)).expect("perspective");
        assert_eq!(forward.slicing, Slicing::Log);
        assert!((forward.near - 0.1).abs() < 1.0e-5, "{}", forward.near);
        assert!((forward.far - 100.0).abs() < 1.0e-2, "{}", forward.far);

        let reversed = Frustum::from_matrix(perspective(100.0, 0.1)).expect("reverse-Z");
        assert!((reversed.near - 0.1).abs() < 1.0e-5, "{}", reversed.near);
        assert!((reversed.far - 100.0).abs() < 1.0e-2, "{}", reversed.far);

        let ortho = Frustum::from_matrix(camera(directx::orthographic(
            -2.0, 2.0, -2.0, 2.0, -5.0, 20.0,
        )))
        .expect("orthographic");
        assert_eq!(ortho.slicing, Slicing::Linear);
        assert_eq!(ortho.near, 0.0);
        assert!((ortho.far - 25.0).abs() < 1.0e-4, "{}", ortho.far);

        let identity = Frustum::from_matrix(Mat4::IDENTITY).expect("identity");
        assert_eq!(identity.slicing, Slicing::Linear);
        assert_eq!((identity.near, identity.far), (0.0, 1.0));
        assert_eq!(identity.size, [GRID_LONG, GRID_SHORT, GRID_SLICES]);

        assert!(Frustum::from_matrix(Mat4::from_cols_array(&[f32::NAN; 16])).is_none());
        let portrait =
            Frustum::from_matrix(directx::perspective(1.0, 0.5, 0.1, 10.0)).expect("portrait");
        assert_eq!(portrait.size, [GRID_SHORT, GRID_LONG, GRID_SLICES]);
    }

    #[test]
    fn a_far_plane_past_f32_still_bins() {
        // A far a hundred million times the near, which rounds 1 - a to 0 in f32.
        let mut seeds = Seeds(23);
        for _ in 0..200 {
            let eye = Vec3::new(seeds.signed(), seeds.signed(), seeds.signed()) * 50.0;
            let toward = Vec3::new(seeds.signed(), seeds.signed(), seeds.signed());
            let view = view::look_at_mat4(eye, eye + toward.normalize(), Vec3::Y);
            let matrix = directx::perspective(1.0, 1.5, 0.01, 1.0e6) * view;
            let frustum = Frustum::from_matrix(matrix).expect("a grid");
            assert!(
                (frustum.near - 0.01).abs() < 1.0e-5,
                "near {}",
                frustum.near
            );
            assert!(frustum.far > 1.0e5, "far {}", frustum.far);
            let grid = frustum.build_grid(40.0);
            assert!(grid.scale.is_finite() && grid.bias.is_finite());
        }
    }

    #[test]
    fn slices_invert() {
        for matrix in [
            perspective(0.1, 100.0),
            camera(directx::orthographic(-2.0, 2.0, -2.0, 2.0, -5.0, 20.0)),
        ] {
            let grid = Frustum::from_matrix(matrix)
                .expect("frustum")
                .build_grid(30.0);
            for slice in [0.0, 1.0, 7.5, 23.0, 24.0] {
                let depth = grid.depth_at(slice);
                assert!((grid.find_slice(depth) - slice).abs() < 1.0e-3, "{slice}");
            }
        }
    }

    /// A point inside a light's reach lands in a cluster of that light's box,
    /// or in the zones that read every light or no bounded light.
    fn check_boxes(matrix: Mat4, centers: &[Vec3], seeds: &mut Seeds) {
        let frustum = Frustum::from_matrix(matrix).expect("frustum");
        let spheres: Vec<Sphere> = centers
            .iter()
            .map(|center| Sphere {
                center: *center,
                radius: 0.5 + 2.0 * seeds.next(),
            })
            .collect();
        let grid = frustum.build_grid(frustum.reach(&spheres));
        for sphere in &spheres {
            let cells = grid.find_cells(sphere);
            for _ in 0..400 {
                let offset = Vec3::new(seeds.signed(), seeds.signed(), seeds.signed());
                let point =
                    sphere.center + offset.normalize_or_zero() * sphere.radius * seeds.next();
                match grid.light_cell(point) {
                    Cell::Every => {}
                    Cell::Global => {
                        panic!("{point} is in reach of {sphere:?} but past {}", grid.reach)
                    }
                    Cell::Cluster(cell) => {
                        let held = cells.is_some_and(|cells| cells.holds(cell));
                        assert!(
                            held,
                            "{point} lands in {cell:?}, outside {cells:?} of {sphere:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn a_point_in_reach_lands_in_its_light_box() {
        let mut seeds = Seeds(7);
        let centers: Vec<Vec3> = (0..64)
            .map(|_| {
                Vec3::new(
                    seeds.signed() * 6.0,
                    seeds.signed() * 4.0,
                    seeds.signed() * 9.0,
                )
            })
            .collect();
        // Lights behind the eye, at it and beside it, included.
        let mut every = centers.clone();
        every.extend([
            Vec3::new(0.0, 1.0, 6.0),
            Vec3::new(0.0, 1.0, 8.0),
            Vec3::new(1.0, 1.0, 5.9),
        ]);
        check_boxes(perspective(0.1, 100.0), &every, &mut seeds);
        check_boxes(perspective(50.0, 0.1), &every, &mut seeds);
        check_boxes(
            camera(directx::orthographic(-4.0, 4.0, -3.0, 3.0, -5.0, 20.0)),
            &every,
            &mut seeds,
        );
        check_boxes(Mat4::IDENTITY, &every, &mut seeds);
    }

    /// Points in the lit part of a spot light: within `range` of the apex and
    /// at most `angle` off the axis.
    fn spot_points(angle: f32, seeds: &mut Seeds) -> Vec<Vec3> {
        (0..10_000)
            .map(|_| {
                let off = angle * seeds.next().sqrt();
                let turn = seeds.next() * std::f32::consts::TAU;
                let direction =
                    Vec3::new(off.sin() * turn.cos(), off.sin() * turn.sin(), off.cos());
                direction * 3.0 * seeds.next().cbrt()
            })
            .collect()
    }

    fn spot(inner: f32, outer: f32) -> GpuLight {
        GpuLight {
            kind: 2,
            intensity: 1.0,
            range: 3.0,
            inner_cone_cos: inner.cos(),
            position: [0.0; 3],
            outer_cone_cos: outer.cos(),
            direction: [0.0, 0.0, 1.0],
            color: [1.0; 3],
        }
    }

    #[test]
    fn sector_spheres_hold_the_lit_cone() {
        let mut seeds = Seeds(11);
        for degrees in [5.0f32, 30.0, 44.0, 46.0, 80.0] {
            let angle = degrees.to_radians();
            let sphere = Sphere::around(&spot(angle * 0.5, angle)).expect("bounded");
            assert!(
                sphere.radius < 3.0,
                "{degrees} degrees should tighten the sphere"
            );
            for point in spot_points(angle, &mut seeds) {
                let distance = point.distance(sphere.center);
                assert!(
                    distance <= sphere.radius * (1.0 + 1.0e-5),
                    "{degrees}: {point} is {distance} from {sphere:?}"
                );
            }
        }
        // An inverted cone lights up to its outer angle with a hard edge.
        let inverted = Sphere::around(&spot(0.6, 0.3)).expect("bounded");
        for point in spot_points(0.3, &mut seeds) {
            assert!(point.distance(inverted.center) <= inverted.radius * (1.0 + 1.0e-5));
        }
        let full = Sphere {
            center: Vec3::ZERO,
            radius: 3.0,
        };
        let mut nan = spot(0.1, 0.2);
        nan.outer_cone_cos = f32::NAN;
        assert_eq!(Sphere::around(&nan), Some(full));
        let mut aimless = spot(0.1, 0.2);
        aimless.direction = [0.0; 3];
        assert_eq!(Sphere::around(&aimless), Some(full));
        for wide in [std::f32::consts::FRAC_PI_2, 2.0, 3.0] {
            let sphere = Sphere::around(&spot(0.1, wide)).expect("bounded");
            assert!(
                (sphere.radius - 3.0).abs() < 1.0e-5 && sphere.center.length() < 1.0e-5,
                "{wide}: {sphere:?}"
            );
        }
    }

    #[test]
    fn lights_without_a_finite_reach_stay_global() {
        let mut point = spot(0.1, 0.2);
        point.kind = 1;
        for range in [0.0, f32::INFINITY] {
            point.range = range;
            assert_eq!(Sphere::around(&point), None);
        }
        point.range = 2.0;
        point.position = [f32::NAN, 0.0, 0.0];
        assert_eq!(Sphere::around(&point), None);
        assert_eq!(Sphere::around(&GpuLight::placeholder()), None);
    }

    #[test]
    fn boxes_pack_into_five_bit_fields() {
        let cells = Cells {
            x: [1, 15],
            y: [0, 8],
            z: [3, 23],
        };
        let packed = cells.pack();
        let field = |index: u32| (packed >> (BOX_BITS * index)) & 31;
        assert_eq!((0..6).map(field).collect::<Vec<_>>(), [1, 15, 0, 8, 3, 23]);
        assert_eq!(cells.count_cells(), 15 * 9 * 21);
    }
}
