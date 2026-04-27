use glam::Vec2;
use crate::color::Color;
use crate::ui_ir::{SurfacePrimitive, ProjectionPolicy, OverlapLaw, SurfaceId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalyzerMode {
    Spectrum,
    Oscilloscope,
    Phase,
}

pub struct Analyzer {
    pub id: SurfaceId,
    pub position: Vec2,
    pub size: Vec2,
    pub mode: AnalyzerMode,
    pub data: Vec<f32>, // Normalized data points (0.0 - 1.0)
    pub colors: AnalyzerColors,
}

pub struct AnalyzerColors {
    pub bg: Color,
    pub grid: Color,
    pub plot: Color,
}

impl Analyzer {
    pub fn new(id: SurfaceId, position: Vec2, size: Vec2, mode: AnalyzerMode) -> Self {
        Self {
            id,
            position,
            size,
            mode,
            data: Vec::new(),
            colors: AnalyzerColors {
                bg: Color::BG_DEEP,
                grid: Color::SYNTAX_COMMENT,
                plot: Color::ACCENT_LIME,
            },
        }
    }

    pub fn build_primitives(&self) -> Vec<SurfacePrimitive> {
        let mut primitives = Vec::new();

        // 1. Background
        primitives.push(SurfacePrimitive::PolyShape {
            id: self.id,
            points: vec![
                [self.position.x, self.position.y],
                [self.position.x + self.size.x, self.position.y],
                [self.position.x + self.size.x, self.position.y + self.size.y],
                [self.position.x, self.position.y + self.size.y],
            ],
            law: OverlapLaw::SemanticOcclusion,
            color: self.colors.bg.to_array(),
        });

        // 2. Data Plot
        match self.mode {
            AnalyzerMode::Spectrum | AnalyzerMode::Oscilloscope => {
                // High-density data should use DensityMap for performance
                primitives.push(SurfacePrimitive::DensityMap {
                    id: self.id,
                    rect: [self.position.x, self.position.y, self.size.x, self.size.y],
                    policy: ProjectionPolicy::HistoricalWindow,
                });
            }
            AnalyzerMode::Phase => {
                // Phase correlation or Goniometer
                primitives.push(SurfacePrimitive::DensityMap {
                    id: self.id,
                    rect: [self.position.x, self.position.y, self.size.x, self.size.y],
                    policy: ProjectionPolicy::Instant,
                });
            }
        }

        primitives
    }
}
