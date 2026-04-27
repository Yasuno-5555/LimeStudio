//! Compiler Lens Overlay — Invisible logic made visible.
//! 
//! ノードの上に浮かび上がる IR / Rust 命令。
//! 信頼と教育のためのオーバーレイ。

use glam::Vec2;
use crate::color::Color;
use crate::ui_ir::{SurfacePrimitive, FrameStyle, TemporalStrategy, GlyphPlacement, SurfaceId};

pub struct CompilerLensOverlay {
    pub id: SurfaceId,
    pub position: Vec2,
    pub ir_lines: Vec<String>,
    pub rust_lines: Vec<String>,
    pub is_active: bool,
}

impl CompilerLensOverlay {
    pub fn new(id: SurfaceId, position: Vec2) -> Self {
        Self {
            id,
            position,
            ir_lines: Vec::new(),
            rust_lines: Vec::new(),
            is_active: false,
        }
    }

    pub fn build_primitives(&self) -> Vec<SurfacePrimitive> {
        let mut primitives = Vec::new();

        if !self.is_active {
            return primitives;
        }

        // 1. Lens Background (Field style)
        primitives.push(SurfacePrimitive::Frame {
            id: self.id,
            rect: [self.position.x, self.position.y, 200.0, 100.0],
            style: FrameStyle::Field,
            color: Color::AUTHORITY_BG.to_array(),
            temporal: TemporalStrategy::Standard(0.1),
        });

        // 2. IR Content (Simplified placeholder)
        primitives.push(SurfacePrimitive::GlyphRun {
            placements: vec![GlyphPlacement {
                glyph_id: 0,
                pos: [self.position.x + 8.0, self.position.y + 20.0],
                scale: 0.9,
            }],
            color: Color::SYNTAX_VAR.to_array(),
        });

        primitives
    }
}
