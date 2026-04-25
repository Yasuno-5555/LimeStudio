//! Lime Surface Input Abstraction
//! 
//! Never bind directly to winit. Use normalized events.

use glam::Vec2;

#[derive(Debug, Clone, Copy)]
pub enum SurfaceEvent {
    PointerMove { position: Vec2 },
    PointerDown { position: Vec2, button: MouseButton },
    PointerUp { position: Vec2, button: MouseButton },
    KeyInput { key: Key, pressed: bool },
    Wheel { delta: Vec2 },
    Focus(bool),
    Resize { width: u32, height: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Shift,
    Control,
    Alt,
    Space,
    Delete,
    Enter,
    Escape,
    // Add more as needed
}

/// Drag Hysteresis Standard
pub struct InteractionState {
    pub drag_origin: Option<Vec2>,
    pub accumulated_delta: Vec2,
    pub is_dragging: bool,
    pub threshold: f32, // Default: 3.0 logical pixels
}

impl InteractionState {
    pub fn new(threshold: f32) -> Self {
        Self {
            drag_origin: None,
            accumulated_delta: Vec2::ZERO,
            is_dragging: false,
            threshold,
        }
    }

    pub fn handle_event(&mut self, event: &SurfaceEvent) {
        match event {
            SurfaceEvent::PointerDown { position, button: MouseButton::Left } => {
                self.drag_origin = Some(*position);
                self.accumulated_delta = Vec2::ZERO;
                self.is_dragging = false;
            }
            SurfaceEvent::PointerMove { position } => {
                if let Some(origin) = self.drag_origin {
                    let delta = *position - origin;
                    self.accumulated_delta = delta;
                    
                    if !self.is_dragging && delta.length() >= self.threshold {
                        self.is_dragging = true;
                    }
                }
            }
            SurfaceEvent::PointerUp { .. } => {
                self.drag_origin = None;
                self.is_dragging = false;
            }
            _ => {}
        }
    }
}
