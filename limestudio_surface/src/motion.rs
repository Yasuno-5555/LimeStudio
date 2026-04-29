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

    /// Critically damped spring update (Implicit Euler for stability)
    /// stiffness: 弾性 (Higher = faster response)
    /// delta_time: 経過時間
    pub fn update(&mut self, target: f32, delta_time: f32, stiffness: f32, damping: f32) -> f32 {
        if delta_time <= 0.0 {
            return self.value;
        }

        let h = delta_time;
        let x_diff = self.value - target;

        // v_{n+1} = v_n + h * (-k * x_{n+1} - c * v_{n+1})
        // x_{n+1} = x_n + h * v_{n+1}
        // Solving for v_{n+1}:
        // v_{n+1} (1 + h*c + h*h*k) = v_n - h*k*x_n

        let det = 1.0 + h * damping + h * h * stiffness;
        let new_v = (self.velocity - h * stiffness * x_diff) / det;
        let new_x = self.value + h * new_v;

        self.velocity = new_v;
        self.value = new_x;
        self.value
    }
}

/// Standard spring constants for Lime Surface (HIG 3.1)
pub mod constants {
    pub const STIFFNESS_STANDARD: f32 = 300.0;
    pub const DAMPING_STANDARD: f32 = 35.0;

    pub const STIFFNESS_FAST: f32 = 600.0;
    pub const DAMPING_FAST: f32 = 50.0;

    pub const STIFFNESS_SLOW: f32 = 150.0;
    pub const DAMPING_SLOW: f32 = 25.0;
}
