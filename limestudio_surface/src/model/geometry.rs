use glam::Vec2;

/// A rectangle defined by its center and size.
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub center: Vec2,
    pub size: Vec2,
}

impl Rect {
    pub fn new(center: Vec2, size: Vec2) -> Self {
        Self { center, size }
    }

    pub fn min(&self) -> Vec2 {
        self.center - self.size * 0.5
    }

    pub fn max(&self) -> Vec2 {
        self.center + self.size * 0.5
    }

    pub fn from_points(p1: Vec2, p2: Vec2) -> Self {
        let min = p1.min(p2);
        let max = p1.max(p2);
        Self {
            center: (min + max) * 0.5,
            size: max - min,
        }
    }

    pub fn intersects(&self, other: Rect) -> bool {
        let s1 = self.size * 0.5;
        let s2 = other.size * 0.5;
        (self.center.x - other.center.x).abs() <= (s1.x + s2.x) &&
        (self.center.y - other.center.y).abs() <= (s1.y + s2.y)
    }

    pub fn contains(&self, point: Vec2) -> bool {
        let half = self.size * 0.5;
        let delta = (point - self.center).abs();
        delta.x <= half.x && delta.y <= half.y
    }

    /// Distance from point to the rectangle.
    /// Negative values are inside (SDF).
    pub fn sdf(&self, point: Vec2) -> f32 {
        let half = self.size * 0.5;
        let d = (point - self.center).abs() - half;
        d.max(Vec2::ZERO).length() + d.x.max(d.y).min(0.0)
    }
}

/// A circle defined by its center and radius.
#[derive(Debug, Clone, Copy)]
pub struct Circle {
    pub center: Vec2,
    pub radius: f32,
}

impl Circle {
    pub fn new(center: Vec2, radius: f32) -> Self {
        Self { center, radius }
    }

    pub fn contains(&self, point: Vec2) -> bool {
        point.distance_squared(self.center) <= self.radius * self.radius
    }

    pub fn sdf(&self, point: Vec2) -> f32 {
        point.distance(self.center) - self.radius
    }
}

/// A line segment defined by two points.
#[derive(Debug, Clone, Copy)]
pub struct Segment {
    pub start: Vec2,
    pub end: Vec2,
}

impl Segment {
    pub fn new(start: Vec2, end: Vec2) -> Self {
        Self { start, end }
    }

    /// Shortest distance from point to the segment.
    pub fn distance_to_point(&self, point: Vec2) -> f32 {
        let pa = point - self.start;
        let ba = self.end - self.start;
        let h = (pa.dot(ba) / ba.dot(ba)).clamp(0.0, 1.0);
        (pa - ba * h).length()
    }

    pub fn sdf(&self, point: Vec2, thickness: f32) -> f32 {
        self.distance_to_point(point) - thickness * 0.5
    }
}
