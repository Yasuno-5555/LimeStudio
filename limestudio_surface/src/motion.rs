//! Lime Surface Motion System
//! 
//! Linear Lerp is Forbidden. 
//! Use critically damped spring motion for all UI interactions.

#[derive(Debug, Clone, Copy)]
pub struct MotionState {
    pub value: f32,
    pub velocity: f32,
}

impl MotionState {
    pub fn new(initial_value: f32) -> Self {
        Self {
            value: initial_value,
            velocity: 0.0,
        }
    }

    /// Critically damped spring update
    /// stiffness: 弾性 (e.g. 150.0)
    /// damping: 減衰 (e.g. 0.8 to 1.0)
    pub fn update(
        &mut self,
        target: f32,
        delta_time: f32,
        stiffness: f32,
        damping: f32,
    ) -> f32 {
        let force = (target - self.value) * stiffness;
        self.velocity = (self.velocity + force * delta_time) * damping;
        self.value += self.velocity * delta_time;
        self.value
    }
}

/// Standard spring constants for Lime Surface
pub mod constants {
    pub const STIFFNESS_FAST: f32 = 400.0;
    pub const STIFFNESS_NORMAL: f32 = 180.0;
    pub const STIFFNESS_SLOW: f32 = 80.0;
    
    pub const DAMPING_CRITICAL: f32 = 0.85; // Roughly critically damped for 60Hz
}
