use glam::Vec2;

pub struct CanvasState {
    pub pan: Vec2,
    pub zoom: f32,
    pub grid_size: f32,
}

impl Default for CanvasState {
    fn default() -> Self { Self::new() }
}

impl CanvasState {
    pub fn new() -> Self {
        Self {
            pan: Vec2::ZERO,
            zoom: 1.0,
            grid_size: 8.0, // The Law of 8px
        }
    }

    pub fn snap(&self, pos: Vec2) -> Vec2 {
        let snapped_x = (pos.x / self.grid_size).round() * self.grid_size;
        let snapped_y = (pos.y / self.grid_size).round() * self.grid_size;
        Vec2::new(snapped_x, snapped_y)
    }

    pub fn screen_to_world(&self, screen_pos: Vec2, viewport_center: Vec2) -> Vec2 {
        (screen_pos - viewport_center) / self.zoom - self.pan
    }

    pub fn world_to_screen(&self, world_pos: Vec2, viewport_center: Vec2) -> Vec2 {
        (world_pos + self.pan) * self.zoom + viewport_center
    }
}
