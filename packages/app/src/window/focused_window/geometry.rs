//! Geometry calculations for window bounds

use super::types::WindowBounds;

impl WindowBounds {
    /// Calculate the area of this rectangle
    #[inline]
    pub const fn area(&self) -> i64 {
        (self.width as i64) * (self.height as i64)
    }

    /// Check if this bounds intersects with another bounds
    #[inline]
    pub fn intersects(&self, other: &WindowBounds) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }

    /// Calculate intersection with another bounds
    #[inline]
    pub fn intersection(&self, other: &WindowBounds) -> Option<WindowBounds> {
        if !self.intersects(other) {
            return None;
        }

        let left = self.x.max(other.x);
        let top = self.y.max(other.y);
        let right = (self.x + self.width).min(other.x + other.width);
        let bottom = (self.y + self.height).min(other.y + other.height);

        Some(WindowBounds::new(left, top, right - left, bottom - top))
    }

    /// Calculate overlap percentage with another bounds (0.0 to 1.0)
    #[inline]
    pub fn overlap_percentage(&self, other: &WindowBounds) -> f64 {
        match self.intersection(other) {
            Some(intersection) => {
                let intersection_area = intersection.area();
                let self_area = self.area();

                if self_area == 0 {
                    0.0
                } else {
                    intersection_area as f64 / self_area as f64
                }
            },
            None => 0.0,
        }
    }
}
