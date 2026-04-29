use glam::Vec2;
use crate::color::Color;
use crate::ui_ir::{SurfacePrimitive, FrameStyle, TemporalStrategy, OverlapLaw, GlyphPlacement, SurfaceId};

pub struct SurfaceListView {
    pub id: SurfaceId,
    pub position: Vec2,
    pub size: Vec2,
    pub items: Vec<String>,
    pub selected_index: Option<usize>,
    pub scroll_offset: f32,
    pub item_height: f32,
    pub colors: ListViewColors,
}

pub struct ListViewColors {
    pub bg: Color,
    pub item_bg: Color,
    pub selection: Color,
    pub text: Color,
}

impl SurfaceListView {
    pub fn new(id: SurfaceId, position: Vec2, size: Vec2) -> Self {
        Self {
            id,
            position,
            size,
            items: Vec::new(),
            selected_index: None,
            scroll_offset: 0.0,
            item_height: 24.0, // 8px grid
            colors: ListViewColors {
                bg: Color::BG_DEEP,
                item_bg: Color::BG_PANEL,
                selection: Color::ACCENT_LIME,
                text: Color::TEXT_PRIMARY,
            },
        }
    }

    pub fn build_primitives(&self) -> Vec<SurfacePrimitive> {
        let mut primitives = Vec::new();

        // 1. Container Background
        primitives.push(SurfacePrimitive::Frame {
            id: self.id,
            rect: [self.position.x, self.position.y, self.size.x, self.size.y],
            style: FrameStyle::Standard,
            color: self.colors.bg.to_array(),
            temporal: TemporalStrategy::Instant,
        });

        // 2. Items with ClipMask
        let mut item_primitives = Vec::new();
        for (i, _item) in self.items.iter().enumerate() {
            let item_y = self.position.y + (i as f32 * self.item_height) - self.scroll_offset;
            let is_selected = self.selected_index == Some(i);

            // Item Background
            item_primitives.push(SurfacePrimitive::Frame {
                id: self.id,
                rect: [self.position.x, item_y, self.size.x, self.item_height],
                style: FrameStyle::Standard,
                color: if is_selected { self.colors.selection.to_array() } else { self.colors.item_bg.to_array() },
                temporal: TemporalStrategy::Fast,
            });

            // Item Text
            item_primitives.push(SurfacePrimitive::GlyphRun {
                placements: vec![GlyphPlacement {
                    glyph_id: 0,
                    pos: [self.position.x + 8.0, item_y + 16.0],
                    scale: 1.0,
                }],
                color: if is_selected { Color::BG_DEEP.to_array() } else { self.colors.text.to_array() },
            });
        }

        primitives.push(SurfacePrimitive::ClipMask {
            id: self.id,
            rect: [self.position.x, self.position.y, self.size.x, self.size.y],
            law: OverlapLaw::SemanticOcclusion,
            children: item_primitives,
        });

        primitives
    }
}
