//! Compiler Lens Overlay — Invisible logic made visible.
//! 
//! ノードの上に浮かび上がる IR / Rust 命令。
//! 信頼と教育のためのオーバーレイ。

use glam::Vec2;
use crate::color::Color;
use crate::scene::SurfaceScene;

pub struct CompilerLensOverlay {
    pub node_id: u32,
    pub position: Vec2,
    pub ir_lines: Vec<String>,
    pub rust_lines: Vec<String>,
    pub alpha: f32, // Fade in/out
}

impl CompilerLensOverlay {
    pub fn new(node_id: u32, position: Vec2) -> Self {
        Self {
            node_id,
            position,
            ir_lines: Vec::new(),
            rust_lines: Vec::new(),
            alpha: 0.0,
        }
    }

    pub fn update_content(&mut self, ir: Vec<String>, rust: String) {
        self.ir_lines = ir;
        self.rust_lines = rust.lines().map(|s| s.to_string()).collect();
    }

    /// UI update loop (Motion System)
    pub fn update(&mut self, dt: f32, is_selected: bool) {
        let target_alpha = if is_selected { 1.0 } else { 0.0 };
        // Use spring motion for alpha? 
        // Trust UI: Precise fade.
        self.alpha += (target_alpha - self.alpha) * (dt / 0.1).clamp(0.0, 1.0);
    }

    /// Add to scene for rendering
    pub fn render(&self, _scene: &mut SurfaceScene) {
        if self.alpha < 0.01 { return; }
        
        // This would create SurfaceOverlay entries in the scene
    }
}
