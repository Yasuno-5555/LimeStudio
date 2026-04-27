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
    pub fn update(
        &mut self,
        target: f32,
        delta_time: f32,
        stiffness: f32,
    ) -> f32 {
        if delta_time <= 0.0 { return self.value; }
        
        // ω (Natural frequency) = sqrt(stiffness)
        // For critical damping, c = 2 * ω
        let omega = stiffness.sqrt();
        let h = delta_time;
        
        // Implicit integration for stability:
        // v_{n+1} = v_n + h * a_{n+1}
        // a_{n+1} = -ω^2 * (x_{n+1} - target) - 2ω * v_{n+1}
        // x_{n+1} = x_n + h * v_{n+1}
        
        let x_diff = self.value - target;
        let det = 1.0 + 2.0 * h * omega + h * h * omega * omega;
        
        let new_v = (self.velocity - h * omega * omega * x_diff) / det;
        let new_x = self.value + h * new_v;
        
        self.velocity = new_v;
        self.value = new_x;
        self.value
    }
}

/// Standard spring constants for Lime Surface
pub mod constants {
    /// 60ms response (ω ≈ 16.6) -> stiffness ≈ 275
    pub const STIFFNESS_NORMAL: f32 = 275.0;
    /// 20ms response (ω ≈ 50.0) -> stiffness ≈ 2500
    pub const STIFFNESS_FAST: f32 = 2500.0;
    /// 150ms response (ω ≈ 6.6) -> stiffness ≈ 44
    pub const STIFFNESS_SLOW: f32 = 44.0;
}
