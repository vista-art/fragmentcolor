#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ImageRegion {
    pub x_min: u32,
    pub y_min: u32,
    pub x_max: u32,
    pub y_max: u32,
}

impl ImageRegion {
    pub fn for_region_i32(x: i32, y: i32, width: i32, height: i32) -> Self {
        let a = (x, y);
        let b = (x.saturating_add(width), y.saturating_add(height));
        let (min, max) = ((a.0.min(b.0), a.1.min(b.1)), (a.0.max(b.0), a.1.max(b.1)));

        Self {
            x_min: min.0.max(0) as u32,
            y_min: min.1.max(0) as u32,
            x_max: max.0.max(0) as u32,
            y_max: max.1.max(0) as u32,
        }
    }

    pub fn for_region(x: u32, y: u32, width: u32, height: u32) -> Self {
        let a = (x, y);
        let b = (x.saturating_add(width), y.saturating_add(height));
        let (min, max) = ((a.0.min(b.0), a.1.min(b.1)), (a.0.max(b.0), a.1.max(b.1)));

        Self {
            x_min: min.0,
            y_min: min.1,
            x_max: max.0,
            y_max: max.1,
        }
    }

    pub fn encompassing_pixels_i32(a: (i32, i32), b: (i32, i32)) -> Self {
        Self::encompassing_pixels(
            (a.0.max(0) as u32, a.1.max(0) as u32),
            (b.0.max(0) as u32, b.1.max(0) as u32),
        )
    }

    pub fn encompassing_pixels(a: (u32, u32), b: (u32, u32)) -> Self {
        // Figure out what our two ranges are
        let (min, max) = ((a.0.min(b.0), a.1.min(b.1)), (a.0.max(b.0), a.1.max(b.1)));

        // Increase max by one pixel as we've calculated the *encompassed* max
        let max = (max.0.saturating_add(1), max.1.saturating_add(1));

        Self {
            x_min: min.0,
            y_min: min.1,
            x_max: max.0,
            y_max: max.1,
        }
    }

    pub fn for_whole_size(width: u32, height: u32) -> Self {
        Self {
            x_min: 0,
            y_min: 0,
            x_max: width,
            y_max: height,
        }
    }

    pub fn for_pixel(x: u32, y: u32) -> Self {
        Self {
            x_min: x,
            y_min: y,
            x_max: x + 1,
            y_max: y + 1,
        }
    }

    pub fn clamp(&mut self, width: u32, height: u32) {
        self.x_min = self.x_min.min(width);
        self.y_min = self.y_min.min(height);
        self.x_max = self.x_max.min(width);
        self.y_max = self.y_max.min(height);
    }

    pub fn union(&mut self, other: ImageRegion) {
        self.x_min = self.x_min.min(other.x_min);
        self.y_min = self.y_min.min(other.y_min);
        self.x_max = self.x_max.max(other.x_max);
        self.y_max = self.y_max.max(other.y_max);
    }

    pub fn encompass(&mut self, x: u32, y: u32) {
        self.x_min = self.x_min.min(x);
        self.y_min = self.y_min.min(y);
        self.x_max = self.x_max.max(x + 1);
        self.y_max = self.y_max.max(y + 1);
    }

    pub fn intersects(&self, other: ImageRegion) -> bool {
        self.x_min <= other.x_max
            && self.x_max >= other.x_min
            && self.y_min <= other.y_max
            && self.y_max >= other.y_min
    }

    pub fn width(&self) -> u32 {
        self.x_max - self.x_min
    }

    pub fn height(&self) -> u32 {
        self.y_max - self.y_min
    }

    /// Clamps this ImageRegion to a theoretical overlap of another ImageRegion,
    /// referring to "overlapping pixels" (such as a copy destination vs copy source),
    /// in such a way that only pixels that are valid for both ImageRegions are valid.
    ///
    /// The other ImageRegion is also clamped to reflect the same overlap.
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
        other: &mut ImageRegion,
    ) {
        // Translate both regions to same coordinate system.

        let r1 = (
            self.x_min as i32,
            self.y_min as i32,
            self.x_max as i32,
            self.y_max as i32,
        );
        let r2 = (
            other.x_min as i32,
            other.y_min as i32,
            other.x_max as i32,
            other.y_max as i32,
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

        self.x_min = r1_result.0 as u32;
        self.y_min = r1_result.1 as u32;
        self.x_max = r1_result.2 as u32;
        self.y_max = r1_result.3 as u32;

        other.x_min = r2_result.0 as u32;
        other.y_min = r2_result.1 as u32;
        other.x_max = r2_result.2 as u32;
        other.y_max = r2_result.3 as u32;
    }
}

#[inline]
fn intersection_same_coordinate_system(
    (r1_x_min, r1_y_min, r1_x_max, r1_y_max): (i32, i32, i32, i32),
    (r2_x_min, r2_y_min, r2_x_max, r2_y_max): (i32, i32, i32, i32),
) -> (i32, i32, i32, i32) {
    // To guard against 'min' being larger than 'max'.
    let r1_x_min = r1_x_min.min(r1_x_max);
    let r1_y_min = r1_y_min.min(r1_y_max);
    let r2_x_min = r2_x_min.min(r2_x_max);
    let r2_y_min = r2_y_min.min(r2_y_max);

    // First part of intersection.
    let r3_x_min = r1_x_min.max(r2_x_min);
    let r3_y_min = r1_y_min.max(r2_y_min);
    let r3_x_max = r1_x_max.min(r2_x_max);
    let r3_y_max = r1_y_max.min(r2_y_max);

    // In case of no overlap.
    let r3_x_min = r3_x_min.min(r3_x_max);
    let r3_y_min = r3_y_min.min(r3_y_max);

    (r3_x_min, r3_y_min, r3_x_max, r3_y_max)
}

#[inline]
fn translate_region(
    (r_x_min, r_y_min, r_x_max, r_y_max): (i32, i32, i32, i32),
    (trans_x, trans_y): (i32, i32),
) -> (i32, i32, i32, i32) {
    (
        r_x_min + trans_x,
        r_y_min + trans_y,
        r_x_max + trans_x,
        r_y_max + trans_y,
    )
}

#[cfg(test)]
mod tests {
    use super::ImageRegion;

    #[test]
    fn clamp_with_intersection() {
        fn test(
            mut a: ImageRegion,
            mut b: ImageRegion,
            a_point: (i32, i32),
            b_point: (i32, i32),
            size: (i32, i32),
            expected_a: ImageRegion,
            expected_b: ImageRegion,
        ) {
            a.clamp_with_intersection(a_point, b_point, size, &mut b);

            assert_eq!(expected_a, a, "a (self) region should match");
            assert_eq!(expected_b, b, "b (other) region should match");
        }

        test(
            ImageRegion::for_whole_size(10, 10),
            ImageRegion::for_whole_size(10, 10),
            (0, 0),
            (0, 0),
            (5, 5),
            ImageRegion::for_region_i32(0, 0, 5, 5),
            ImageRegion::for_region_i32(0, 0, 5, 5),
        );

        test(
            ImageRegion::for_whole_size(10, 10),
            ImageRegion::for_whole_size(150, 150),
            (-1, -1),
            (100, 100),
            (5, 5),
            ImageRegion::for_region_i32(0, 0, 4, 4),
            ImageRegion::for_region_i32(101, 101, 4, 4),
        );

        test(
            ImageRegion::for_whole_size(10, 10),
            ImageRegion::for_whole_size(150, 150),
            (-1, -1),
            (100, 100),
            (15, 15),
            ImageRegion::for_region_i32(0, 0, 10, 10),
            ImageRegion::for_region_i32(101, 101, 10, 10),
        );

        test(
            ImageRegion::for_region(10, 10, 20, 20),
            ImageRegion::for_whole_size(150, 150),
            (15, 5),
            (0, 0),
            (15, 15),
            ImageRegion::for_region_i32(15, 10, 15, 10),
            ImageRegion::for_region_i32(0, 5, 15, 10),
        );

        test(
            ImageRegion::for_whole_size(800, 600),
            ImageRegion::for_whole_size(200, 40),
            (400, 440),
            (40, 0),
            (40, 40),
            ImageRegion::for_region_i32(400, 440, 40, 40),
            ImageRegion::for_region_i32(40, 0, 40, 40),
        );

        test(
            ImageRegion::for_whole_size(240, 180),
            ImageRegion::for_whole_size(238, 164),
            (-1, 0),
            (0, 0),
            (240, 180),
            ImageRegion::for_region_i32(0, 0, 237, 164),
            ImageRegion::for_region_i32(1, 0, 237, 164),
        );

        test(
            ImageRegion::for_whole_size(10, 10),
            ImageRegion::for_whole_size(10, 10),
            (15, 0),
            (0, 15),
            (100, 100),
            ImageRegion::for_region_i32(0, 0, 0, 0),
            ImageRegion::for_region_i32(0, 0, 0, 0),
        );
    }
}
