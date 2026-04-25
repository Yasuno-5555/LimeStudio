pub mod motion;
pub mod color;
pub mod input;
pub mod scene;
pub mod render;
pub mod widgets;
pub mod profiler;
pub mod host_attach;
pub mod runtime;

/// The Core Surface Engine
pub struct SurfaceEngine {
    pub scene: scene::SurfaceScene,
    pub input: input::InteractionState,
    pub profiler: profiler::FrameProfiler,
    pub canvas: widgets::canvas::GraphCanvas,
}

impl SurfaceEngine {
    pub fn new() -> Self {
        Self {
            scene: scene::SurfaceScene::new(),
            input: input::InteractionState::new(3.0),
            profiler: profiler::FrameProfiler::new(),
            canvas: widgets::canvas::GraphCanvas::new(),
        }
    }
}
