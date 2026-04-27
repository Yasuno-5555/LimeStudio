use glam::{Vec2, Mat4};

#[derive(Debug, Clone)]
pub struct InfiniteCamera {
    /// Center of the camera in world space.
    pub center: Vec2,
    /// Zoom level (1.0 is default).
    pub zoom: f32,
    /// Logical size of the viewport.
    pub viewport_size: Vec2,
}

impl Default for InfiniteCamera {
    fn default() -> Self {
        Self {
            center: Vec2::ZERO,
            zoom: 1.0,
            viewport_size: Vec2::new(1.0, 1.0),
        }
    }
}

impl InfiniteCamera {
    pub fn new(viewport_size: Vec2) -> Self {
        Self {
            center: Vec2::ZERO,
            zoom: 1.0,
            viewport_size,
        }
    }

    /// Convert screen coordinates to world coordinates.
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        let half_size = self.viewport_size * 0.5;
        // Normalize screen_pos to [-1, 1] relative to center
        let normalized = (screen_pos - half_size) / half_size;
        // Invert Y because screen Y is down
        let _normalized = Vec2::new(normalized.x, -normalized.y);
        
        // This is a bit simplified. Let's do it more directly.
        // Screen Space (0,0 to width,height)
        // World Space center at self.center
        
        let rel_screen = screen_pos - half_size;
        self.center + rel_screen / self.zoom
    }

    /// Convert world coordinates to screen coordinates.
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        let half_size = self.viewport_size * 0.5;
        let rel_world = world_pos - self.center;
        half_size + rel_world * self.zoom
    }

    /// Get the View-Projection matrix for wgpu.
    /// Maps World Space to NDC [-1, 1].
    pub fn view_projection(&self) -> Mat4 {
        let half_size = self.viewport_size * 0.5;
        let left = self.center.x - half_size.x / self.zoom;
        let right = self.center.x + half_size.x / self.zoom;
        let bottom = self.center.y + half_size.y / self.zoom; // Screen Y is down, so top/bottom might be flipped in world
        let top = self.center.y - half_size.y / self.zoom;
        
        Mat4::orthographic_rh(left, right, bottom, top, 0.0, 1000.0)
    }
    
    pub fn pan(&mut self, delta: Vec2) {
        self.center -= delta / self.zoom;
    }
    
    pub fn zoom_at(&mut self, screen_pos: Vec2, delta: f32) {
        let _old_zoom = self.zoom;
        let world_at_mouse = self.screen_to_world(screen_pos);
        
        self.zoom *= delta;
        self.zoom = self.zoom.clamp(0.01, 100.0);
        
        // Adjust center to keep world_at_mouse under the cursor
        let new_world_at_mouse = self.screen_to_world(screen_pos);
        self.center += world_at_mouse - new_world_at_mouse;
    }
}
