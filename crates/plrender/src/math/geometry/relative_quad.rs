use crate::Vec2;
use crate::Vec4;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct RelativeQuad {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

impl Default for RelativeQuad {
    fn default() -> Self {
        Self {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 1.0,
            max_y: 1.0,
        }
    }
}

// Adapted from Ruffle's PixelRegion
impl RelativeQuad {
    pub fn from_region(x: f32, y: f32, width: f32, height: f32) -> Self {
        let a = (x, y);
        let b = (x + width, y + height);
        let (min, max) = ((a.0.min(b.0), a.1.min(b.1)), (a.0.max(b.0), a.1.max(b.1)));

        Self {
            min_x: min.0.max(0.0),
            min_y: min.1.max(0.0),
            max_x: max.0.max(0.0),
            max_y: max.1.max(0.0),
        }
    }

    pub fn from_tuple(size: (f32, f32)) -> Self {
        Self::from_tuples((0.0, 0.0), size)
    }

    pub fn from_tuples(a: (f32, f32), b: (f32, f32)) -> Self {
        let (min, max) = ((a.0.min(b.0), a.1.min(b.1)), (a.0.max(b.0), a.1.max(b.1)));

        Self {
            min_x: min.0,
            min_y: min.1,
            max_x: max.0,
            max_y: max.1,
        }
    }

    pub fn from_arrays(a: [f32; 2], b: [f32; 2]) -> Self {
        Self::from_tuples((a[0], a[1]), (b[0], b[1]))
    }

    pub fn to_range(&self) -> std::ops::Range<mint::Point2<f32>> {
        let begin = mint::Point2 {
            x: self.min_x as f32,
            y: self.min_y as f32,
        };

        let end = mint::Point2 {
            x: self.max_x as f32,
            y: self.max_y as f32,
        };

        begin..end
    }

    pub fn to_array(&self) -> [f32; 4] {
        [
            self.min_x as f32,
            self.min_y as f32,
            self.max_x as f32,
            self.max_y as f32,
        ]
    }

    pub fn from_size(width: f32, height: f32) -> Self {
        Self {
            min_x: 0.0,
            min_y: 0.0,
            max_x: width,
            max_y: height,
        }
    }

    pub fn from_size_f32(width: f32, height: f32) -> Self {
        Self {
            min_x: 0.0,
            min_y: 0.0,
            max_x: width as f32,
            max_y: height as f32,
        }
    }

    pub fn clamp(&mut self, width: f32, height: f32) {
        self.min_x = self.min_x.min(width);
        self.min_y = self.min_y.min(height);
        self.max_x = self.max_x.min(width);
        self.max_y = self.max_y.min(height);
    }

    pub fn union(&mut self, other: RelativeQuad) {
        self.min_x = self.min_x.min(other.min_x);
        self.min_y = self.min_y.min(other.min_y);
        self.max_x = self.max_x.max(other.max_x);
        self.max_y = self.max_y.max(other.max_y);
    }

    pub fn intersects(&self, other: RelativeQuad) -> bool {
        self.min_x <= other.max_x
            && self.max_x >= other.min_x
            && self.min_y <= other.max_y
            && self.max_y >= other.min_y
    }

    pub fn area(&self) -> f32 {
        self.width() * self.height()
    }

    pub fn antialias_factor(&self) -> f32 {
        2.0 / self.smaller_side() as f32
    }

    pub fn smaller_side(&self) -> f32 {
        self.width().min(self.height())
    }

    pub fn larger_side(&self) -> f32 {
        self.width().max(self.height())
    }

    pub fn is_larger_than(&self, other: RelativeQuad) -> bool {
        self.area() > other.area()
    }

    pub fn is_smaller_than(&self, other: RelativeQuad) -> bool {
        self.area() < other.area()
    }

    pub fn equals(&self, other: RelativeQuad) -> bool {
        self.min_x == other.min_x
            && self.min_y == other.min_y
            && self.max_x == other.max_x
            && self.max_y == other.max_y
    }

    pub fn width(&self) -> f32 {
        self.max_x - self.min_x
    }

    pub fn height(&self) -> f32 {
        self.max_y - self.min_y
    }

    pub fn half_width(&self) -> f32 {
        self.width() / 2.0
    }

    pub fn half_height(&self) -> f32 {
        self.height() / 2.0
    }

    pub fn outbound_radius(&self) -> f32 {
        let width = self.half_width();
        let height = self.half_height();
        (width * width + height * height).sqrt()
    }

    pub fn inbound_radius(&self) -> f32 {
        self.half_width().min(self.half_height())
    }

    pub fn aspect(&self) -> f32 {
        if self.height() == 0.0 {
            return 0.0;
        }
        self.width() as f32 / self.height() as f32
    }

    pub fn pixel_center(&self) -> (f32, f32) {
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
        Vec4 {
            x: self.min_x as f32,
            y: self.min_y as f32,
            z: self.max_x as f32,
            w: self.max_y as f32,
        }
    }

    pub fn center_f32(&self) -> Vec2 {
        Vec2 {
            x: self.min_x as f32 + self.width() as f32 / 2.0,
            y: self.min_y as f32 + self.height() as f32 / 2.0,
        }
    }

    /// Clamps this RelativeQuad to a theoretical overlap of another RelativeQuad,
    /// referring to "overlapping pixels" (such as a copy destination vs copy source),
    /// in such a way that only pixels that are valid for both RelativeQuads are valid.
    ///
    /// The other RelativeQuad is also clamped to reflect the same overlap.
    ///
    /// The overlap of two regions starts at `self_point` on `self`, and `other_point` on `other`,
    /// and is at most `size` big.
    ///
    /// The overlap does not actually need to happen on the same coordinate plane,
    /// for example -1,-1 on this may be 100,100 on other, with an overlap region of 5x5.
    /// As long as both textures can fit that, that's considered an overlap.
    /// However, since -1,-1 is outside of the valid area on the first region,
    /// the overlap actually happens at 0.0,0 and 101,101 for a size of 4x4.
    pub fn clamp_with_intersection(
        &mut self,
        self_point: (f32, f32),
        other_point: (f32, f32),
        size: (f32, f32),
        other: &mut RelativeQuad,
    ) {
        // Translate both regions to same coordinate system.

        let r1 = (
            self.min_x as f32,
            self.min_y as f32,
            self.max_x as f32,
            self.max_y as f32,
        );
        let r2 = (
            other.min_x as f32,
            other.min_y as f32,
            other.max_x as f32,
            other.max_y as f32,
        );

        let r1_trans = translate_region(r1, (-self_point.0, -self_point.1));
        let r2_trans = translate_region(r2, (-other_point.0, -other_point.1));

        // Intersection.

        let inters = intersection_same_coordinate_system(
            intersection_same_coordinate_system(r1_trans, r2_trans),
            (0.0, 0.0, size.0, size.1),
        );

        // Translate the intersection back.

        let r1_result = translate_region(inters, self_point);
        let r2_result = translate_region(inters, other_point);

        // Ensure empty results yield (0, 0.0, 0.0, 0).

        let is_empty = inters.0 == inters.2 || inters.1 == inters.3;

        let r1_result = if is_empty {
            (0.0, 0.0, 0.0, 0.0)
        } else {
            r1_result
        };
        let r2_result = if is_empty {
            (0.0, 0.0, 0.0, 0.0)
        } else {
            r2_result
        };

        // Mutate.

        self.min_x = r1_result.0 as f32;
        self.min_y = r1_result.1 as f32;
        self.max_x = r1_result.2 as f32;
        self.max_y = r1_result.3 as f32;

        other.min_x = r2_result.0 as f32;
        other.min_y = r2_result.1 as f32;
        other.max_x = r2_result.2 as f32;
        other.max_y = r2_result.3 as f32;
    }
}

#[inline]
fn intersection_same_coordinate_system(
    (r1_min_x, r1_min_y, r1_max_x, r1_max_y): (f32, f32, f32, f32),
    (r2_min_x, r2_min_y, r2_max_x, r2_max_y): (f32, f32, f32, f32),
) -> (f32, f32, f32, f32) {
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
    (r_min_x, r_min_y, r_max_x, r_max_y): (f32, f32, f32, f32),
    (trans_x, trans_y): (f32, f32),
) -> (f32, f32, f32, f32) {
    (
        r_min_x + trans_x,
        r_min_y + trans_y,
        r_max_x + trans_x,
        r_max_y + trans_y,
    )
}

#[cfg(test)]
mod tests {
    use super::RelativeQuad;

    #[test]
    fn clamp_with_intersection() {
        fn test(
            mut a: RelativeQuad,
            mut b: RelativeQuad,
            a_point: (f32, f32),
            b_point: (f32, f32),
            size: (f32, f32),
            expected_a: RelativeQuad,
            expected_b: RelativeQuad,
        ) {
            a.clamp_with_intersection(a_point, b_point, size, &mut b);

            assert_eq!(expected_a, a, "a (self) region should match");
            assert_eq!(expected_b, b, "b (other) region should match");
        }

        test(
            RelativeQuad::from_size(10.0, 10.0),
            RelativeQuad::from_size(10.0, 10.0),
            (0.0, 0.0),
            (0.0, 0.0),
            (5.0, 5.0),
            RelativeQuad::from_region(0.0, 0.0, 5.0, 5.0),
            RelativeQuad::from_region(0.0, 0.0, 5.0, 5.0),
        );

        test(
            RelativeQuad::from_size(10.0, 10.0),
            RelativeQuad::from_size(15.0, 150.0),
            (-1.0, -1.0),
            (10.0, 10.0),
            (5.0, 5.0),
            RelativeQuad::from_region(0.0, 0.0, 4.0, 4.0),
            RelativeQuad::from_region(1.0, 1.0, 4.0, 4.0),
        );

        test(
            RelativeQuad::from_size(10.0, 10.0),
            RelativeQuad::from_size(15.00, 150.0),
            (-1.0, -1.0),
            (10.0, 10.0),
            (15.0, 15.0),
            RelativeQuad::from_region(0.0, 0.0, 10.0, 10.0),
            RelativeQuad::from_region(1.001, 101.0, 10.0, 10.0),
        );

        test(
            RelativeQuad::from_region(10.0, 10.0, 20.0, 20.0),
            RelativeQuad::from_size(15.00, 150.0),
            (15.0, 5.0),
            (0.0, 0.0),
            (15.0, 15.0),
            RelativeQuad::from_region(1.05, 10.0, 15.0, 10.0),
            RelativeQuad::from_region(0.0, 5.0, 15.0, 10.0),
        );

        test(
            RelativeQuad::from_size(80.00, 60.0),
            RelativeQuad::from_size(20.00, 40.0),
            (40.0, 440.0),
            (40.0, 0.0),
            (40.0, 40.0),
            RelativeQuad::from_region(4.000, 440.0, 40.0, 40.0),
            RelativeQuad::from_region(4.00, 0.0, 40.0, 40.0),
        );

        test(
            RelativeQuad::from_size(24.00, 180.0),
            RelativeQuad::from_size(23.08, 164.0),
            (-1.0, 0.0),
            (0.0, 0.0),
            (240.0, 180.0),
            RelativeQuad::from_region(0.0, 0.0, 237.0, 164.0),
            RelativeQuad::from_region(1.0, 0.0, 237.0, 164.0),
        );

        test(
            RelativeQuad::from_size(10.0, 10.0),
            RelativeQuad::from_size(10.0, 10.0),
            (15.0, 0.0),
            (0.0, 15.0),
            (10.0, 10.0),
            RelativeQuad::from_region(0.0, 0.0, 0.0, 0.0),
            RelativeQuad::from_region(0.0, 0.0, 0.0, 0.0),
        );
    }
}
