//! Frame Profiler Overlay — Trust UI for the UI.
//! 
//! Transparent reporting of rendering costs.

use std::time::{Instant, Duration};

pub struct FrameProfiler {
    pub start_time: Instant,
    pub last_frame_time: Duration,
    pub ui_logic_time: Duration,
    pub gpu_render_time: Duration,
    pub text_layout_time: Duration,
    pub frames: Vec<f32>, // Rolling history for graph
}

impl Default for FrameProfiler {
    fn default() -> Self { Self::new() }
}

impl FrameProfiler {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            last_frame_time: Duration::ZERO,
            ui_logic_time: Duration::ZERO,
            gpu_render_time: Duration::ZERO,
            text_layout_time: Duration::ZERO,
            frames: Vec::with_capacity(60),
        }
    }

    pub fn begin_frame(&mut self) {
        self.start_time = Instant::now();
    }

    pub fn end_frame(&mut self) {
        self.last_frame_time = self.start_time.elapsed();
        if self.frames.len() >= 60 {
            self.frames.remove(0);
        }
        self.frames.push(self.last_frame_time.as_secs_f32() * 1000.0);
    }

    pub fn report(&self) {
        println!("═══ Frame Profiler ═══");
        println!("  UI Logic:    {:.2}ms", self.ui_logic_time.as_secs_f32() * 1000.0);
        println!("  Text Layout: {:.2}ms", self.text_layout_time.as_secs_f32() * 1000.0);
        println!("  GPU Render:  {:.2}ms", self.gpu_render_time.as_secs_f32() * 1000.0);
        println!("  Total Frame: {:.2}ms", self.last_frame_time.as_secs_f32() * 1000.0);
        
        if self.last_frame_time.as_secs_f32() > 0.0166 {
            println!("  [DANGER] Frame dropped (slow render)");
        }
        println!("══════════════════════");
    }
}
