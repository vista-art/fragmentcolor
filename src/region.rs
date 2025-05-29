use glam::Vec2;
use glam::Vec4;

use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "python", pyo3::pyclass)]
/// A region in 2D space designed to handle viewport and texture regions
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Region {
    pub min_x: u32,
    pub min_y: u32,
    pub max_x: u32,
    pub max_y: u32,
}

impl Default for Region {
    fn default() -> Self {
        Self {
            min_x: 0,
            min_y: 0,
            max_x: 1,
            max_y: 1,
        }
    }
}

impl From<wgpu::Extent3d> for Region {
    fn from(e: wgpu::Extent3d) -> Self {
        Self::from_size(e.width, e.height)
    }
}

impl From<&winit::dpi::PhysicalSize<u32>> for Region {
    fn from(s: &winit::dpi::PhysicalSize<u32>) -> Self {
        Self::from_size(s.width, s.height)
    }
}

impl Region {
    pub fn new(origin: impl Into<(u32, u32)>, size: impl Into<(u32, u32)>) -> Self {
        let (x, y) = origin.into();
        let (w, h) = size.into();
        Self::from_region(x, y, w, h)
    }

    pub fn from_region_i32(x: i32, y: i32, width: i32, height: i32) -> Self {
        let a = (x, y);
        let b = (x.saturating_add(width), y.saturating_add(height));
        let (min, max) = ((a.0.min(b.0), a.1.min(b.1)), (a.0.max(b.0), a.1.max(b.1)));

        Self {
            min_x: min.0.max(0) as u32,
            min_y: min.1.max(0) as u32,
            max_x: max.0.max(0) as u32,
            max_y: max.1.max(0) as u32,
        }
    }

    pub fn from_region(x: u32, y: u32, width: u32, height: u32) -> Self {
        let a: (u32, u32) = (x, y);
        let b = (x.saturating_add(width), y.saturating_add(height));
        let (min, max) = ((a.0.min(b.0), a.1.min(b.1)), (a.0.max(b.0), a.1.max(b.1)));

        Self {
            min_x: min.0,
            min_y: min.1,
            max_x: max.0,
            max_y: max.1,
        }
    }

    pub fn from_tuples_i32(a: (i32, i32), b: (i32, i32)) -> Self {
        Self::from_tuples(
            (a.0.max(0) as u32, a.1.max(0) as u32),
            (b.0.max(0) as u32, b.1.max(0) as u32),
        )
    }

    pub fn from_tuple(size: (u32, u32)) -> Self {
        Self::from_tuples((0, 0), size)
    }

    pub fn from_tuples(a: (u32, u32), b: (u32, u32)) -> Self {
        // Figure out what our two ranges are
        let (min, max) = ((a.0.min(b.0), a.1.min(b.1)), (a.0.max(b.0), a.1.max(b.1)));

        // Increase max by one pixel as we've calculated the *encompassed* max
        let max = (max.0.saturating_add(1), max.1.saturating_add(1));

        Self {
            min_x: min.0,
            min_y: min.1,
            max_x: max.0,
            max_y: max.1,
        }
    }

    pub fn from_arrays_i32(a: [i32; 2], b: [i32; 2]) -> Self {
        Self::from_tuples_i32((a[0], a[1]), (b[0], b[1]))
    }

    pub fn to_array(&self) -> [f32; 4] {
        [
            self.min_x as f32,
            self.min_y as f32,
            self.max_x as f32,
            self.max_y as f32,
        ]
    }

    pub fn from_wgpu_size(size: wgpu::Extent3d) -> Self {
        Self {
            min_x: 0,
            min_y: 0,
            max_x: size.width,
            max_y: size.height,
        }
    }

    pub fn to_wgpu_size(&self) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: self.width(),
            height: self.height(),
            depth_or_array_layers: 1,
        }
    }

    pub fn from_window_size(size: &winit::dpi::PhysicalSize<u32>) -> Self {
        Self {
            min_x: 0,
            min_y: 0,
            max_x: size.width,
            max_y: size.height,
        }
    }

    pub fn from_window_logical_size(size: &winit::dpi::LogicalSize<u32>) -> Self {
        Self {
            min_x: 0,
            min_y: 0,
            max_x: size.width,
            max_y: size.height,
        }
    }

    pub fn from_size(width: u32, height: u32) -> Self {
        Self {
            min_x: 0,
            min_y: 0,
            max_x: width,
            max_y: height,
        }
    }

    pub fn from_size_f32(width: f32, height: f32) -> Self {
        Self {
            min_x: 0,
            min_y: 0,
            max_x: width as u32,
            max_y: height as u32,
        }
    }

    pub fn from_pixel(x: u32, y: u32) -> Self {
        Self {
            min_x: x,
            min_y: y,
            max_x: x + 1,
            max_y: y + 1,
        }
    }

    pub fn clamp(&mut self, width: u32, height: u32) {
        self.min_x = self.min_x.min(width);
        self.min_y = self.min_y.min(height);
        self.max_x = self.max_x.min(width);
        self.max_y = self.max_y.min(height);
    }

    pub fn union(&mut self, other: Region) {
        self.min_x = self.min_x.min(other.min_x);
        self.min_y = self.min_y.min(other.min_y);
        self.max_x = self.max_x.max(other.max_x);
        self.max_y = self.max_y.max(other.max_y);
    }

    pub fn encompass(&mut self, x: u32, y: u32) {
        self.min_x = self.min_x.min(x);
        self.min_y = self.min_y.min(y);
        self.max_x = self.max_x.max(x + 1);
        self.max_y = self.max_y.max(y + 1);
    }

    pub fn intersects(&self, other: Region) -> bool {
        self.min_x <= other.max_x
            && self.max_x >= other.min_x
            && self.min_y <= other.max_y
            && self.max_y >= other.min_y
    }

    pub fn area(&self) -> u32 {
        self.width() * self.height()
    }

    pub fn antialias_factor(&self) -> f32 {
        2.0 / self.smaller_side() as f32
    }

    pub fn smaller_side(&self) -> u32 {
        self.width().min(self.height())
    }

    pub fn larger_side(&self) -> u32 {
        self.width().max(self.height())
    }

    pub fn is_larger_than(&self, other: Region) -> bool {
        self.area() > other.area()
    }

    pub fn is_smaller_than(&self, other: Region) -> bool {
        self.area() < other.area()
    }

    pub fn equals(&self, other: Region) -> bool {
        self.min_x == other.min_x
            && self.min_y == other.min_y
            && self.max_x == other.max_x
            && self.max_y == other.max_y
    }

    pub fn width(&self) -> u32 {
        u32::abs_diff(self.max_x, self.min_x)
    }

    pub fn height(&self) -> u32 {
        u32::abs_diff(self.max_y, self.min_y)
    }

    pub fn width_f32(&self) -> f32 {
        self.width() as f32
    }

    pub fn height_f32(&self) -> f32 {
        self.height() as f32
    }

    pub fn half_width(&self) -> u32 {
        self.width() / 2
    }

    pub fn half_height(&self) -> u32 {
        self.height() / 2
    }

    pub fn half_width_f32(&self) -> f32 {
        self.width() as f32 / 2.0
    }

    pub fn half_height_f32(&self) -> f32 {
        self.height() as f32 / 2.0
    }

    pub fn outbound_radius(&self) -> f32 {
        let width = self.half_width_f32();
        let height = self.half_height_f32();
        (width * width + height * height).sqrt()
    }

    pub fn inbound_radius(&self) -> f32 {
        self.half_width_f32().min(self.half_height_f32())
    }

    pub fn from_inbound_radius(radius: f32) -> Self {
        Self {
            min_x: 0,
            min_y: 0,
            max_x: (radius * 2.0) as u32,
            max_y: (radius * 2.0) as u32,
        }
    }

    pub fn aspect(&self) -> f32 {
        if self.height() == 0 {
            return 0.0;
        }
        self.width() as f32 / self.height() as f32
    }

    pub fn pixel_center(&self) -> (u32, u32) {
        (
            self.min_x + self.half_width(),
            self.min_y + self.half_height(),
        )
    }

    pub fn to_vec2(&self) -> Vec2 {
        Vec2 {
            x: self.width() as f32,
            y: self.height() as f32,
        }
    }

    pub fn to_vec4(&self) -> Vec4 {
        Vec4::new(
            self.min_x as f32,
            self.min_y as f32,
            self.max_x as f32,
            self.max_y as f32,
        )
    }

    pub fn center_f32(&self) -> Vec2 {
        Vec2 {
            x: self.min_x as f32 + self.width() as f32 / 2.0,
            y: self.min_y as f32 + self.height() as f32 / 2.0,
        }
    }

    /// Clamps this Region to a theoretical overlap of another Region,
    /// referring to "overlapping pixels" (such as a copy destination vs copy source),
    /// in such a way that only pixels that are valid for both Regions are valid.
    ///
    /// The other Region is also clamped to reflect the same overlap.
    ///
    /// The overlap of two regions starts at `self_point` on `self`, and `other_point` on `other`,
    /// and is at most `size` big.
    ///
    /// The overlap does not actually need to happen on the same coordinate plane,
    /// for example -1,-1 on this may be 100,100 on other, with an overlap region of 5x5.
    /// As long as both textures can fit that, that's considered an overlap.
    /// However, since -1,-1 is outside of the valid area on the first region,
    /// the overlap actually happens at 0,0 and 101,101 for a size of 4x4.
    pub fn clamp_with_intersection(
        &mut self,
        self_point: (i32, i32),
        other_point: (i32, i32),
        size: (i32, i32),
        other: &mut Region,
    ) {
        // Translate both regions to same coordinate system.

        let r1 = (
            self.min_x as i32,
            self.min_y as i32,
            self.max_x as i32,
            self.max_y as i32,
        );
        let r2 = (
            other.min_x as i32,
            other.min_y as i32,
            other.max_x as i32,
            other.max_y as i32,
        );

        let r1_trans = translate_region(r1, (-self_point.0, -self_point.1));
        let r2_trans = translate_region(r2, (-other_point.0, -other_point.1));

        // Intersection.

        let inters = intersection_same_coordinate_system(
            intersection_same_coordinate_system(r1_trans, r2_trans),
            (0, 0, size.0, size.1),
        );

        // Translate the intersection back.

        let r1_result = translate_region(inters, self_point);
        let r2_result = translate_region(inters, other_point);

        // Ensure empty results yield (0, 0, 0, 0).

        let is_empty = inters.0 == inters.2 || inters.1 == inters.3;

        let r1_result = if is_empty { (0, 0, 0, 0) } else { r1_result };
        let r2_result = if is_empty { (0, 0, 0, 0) } else { r2_result };

        // Mutate.

        self.min_x = r1_result.0 as u32;
        self.min_y = r1_result.1 as u32;
        self.max_x = r1_result.2 as u32;
        self.max_y = r1_result.3 as u32;

        other.min_x = r2_result.0 as u32;
        other.min_y = r2_result.1 as u32;
        other.max_x = r2_result.2 as u32;
        other.max_y = r2_result.3 as u32;
    }
}

#[inline]
fn intersection_same_coordinate_system(
    (r1_min_x, r1_min_y, r1_max_x, r1_max_y): (i32, i32, i32, i32),
    (r2_min_x, r2_min_y, r2_max_x, r2_max_y): (i32, i32, i32, i32),
) -> (i32, i32, i32, i32) {
    // To guard against 'min' being larger than 'max'.
    let r1_min_x = r1_min_x.min(r1_max_x);
    let r1_min_y = r1_min_y.min(r1_max_y);
    let r2_min_x = r2_min_x.min(r2_max_x);
    let r2_min_y = r2_min_y.min(r2_max_y);

    // First part of intersection.
    let r3_min_x = r1_min_x.max(r2_min_x);
    let r3_min_y = r1_min_y.max(r2_min_y);
    let r3_max_x = r1_max_x.min(r2_max_x);
    let r3_max_y = r1_max_y.min(r2_max_y);

    // In case of no overlap.
    let r3_min_x = r3_min_x.min(r3_max_x);
    let r3_min_y = r3_min_y.min(r3_max_y);

    (r3_min_x, r3_min_y, r3_max_x, r3_max_y)
}

#[inline]
fn translate_region(
    (r_min_x, r_min_y, r_max_x, r_max_y): (i32, i32, i32, i32),
    (trans_x, trans_y): (i32, i32),
) -> (i32, i32, i32, i32) {
    (
        r_min_x + trans_x,
        r_min_y + trans_y,
        r_max_x + trans_x,
        r_max_y + trans_y,
    )
}

#[cfg(test)]
mod tests {
    use super::Region;

    #[test]
    fn clamp_with_intersection() {
        fn test(
            mut a: Region,
            mut b: Region,
            a_point: (i32, i32),
            b_point: (i32, i32),
            size: (i32, i32),
            expected_a: Region,
            expected_b: Region,
        ) {
            a.clamp_with_intersection(a_point, b_point, size, &mut b);

            assert_eq!(expected_a, a, "a (self) region should match");
            assert_eq!(expected_b, b, "b (other) region should match");
        }

        test(
            Region::from_size(10, 10),
            Region::from_size(10, 10),
            (0, 0),
            (0, 0),
            (5, 5),
            Region::from_region_i32(0, 0, 5, 5),
            Region::from_region_i32(0, 0, 5, 5),
        );

        test(
            Region::from_size(10, 10),
            Region::from_size(150, 150),
            (-1, -1),
            (100, 100),
            (5, 5),
            Region::from_region_i32(0, 0, 4, 4),
            Region::from_region_i32(101, 101, 4, 4),
        );

        test(
            Region::from_size(10, 10),
            Region::from_size(150, 150),
            (-1, -1),
            (100, 100),
            (15, 15),
            Region::from_region_i32(0, 0, 10, 10),
            Region::from_region_i32(101, 101, 10, 10),
        );

        test(
            Region::from_region(10, 10, 20, 20),
            Region::from_size(150, 150),
            (15, 5),
            (0, 0),
            (15, 15),
            Region::from_region_i32(15, 10, 15, 10),
            Region::from_region_i32(0, 5, 15, 10),
        );

        test(
            Region::from_size(800, 600),
            Region::from_size(200, 40),
            (400, 440),
            (40, 0),
            (40, 40),
            Region::from_region_i32(400, 440, 40, 40),
            Region::from_region_i32(40, 0, 40, 40),
        );

        test(
            Region::from_size(240, 180),
            Region::from_size(238, 164),
            (-1, 0),
            (0, 0),
            (240, 180),
            Region::from_region_i32(0, 0, 237, 164),
            Region::from_region_i32(1, 0, 237, 164),
        );

        test(
            Region::from_size(10, 10),
            Region::from_size(10, 10),
            (15, 0),
            (0, 15),
            (100, 100),
            Region::from_region_i32(0, 0, 0, 0),
            Region::from_region_i32(0, 0, 0, 0),
        );
    }
}
