/// 線形補間スムーザー (パラメータの急激な変化を抑制する)
pub struct Smoother {
    current_value: f32,
    target_value: f32,
    step_size: f32,
    sample_rate: f32,
    ramp_time_ms: f32,
}

impl Smoother {
    pub fn new(initial_value: f32, sample_rate: f32, ramp_time_ms: f32) -> Self {
        Self {
            current_value: initial_value,
            target_value: initial_value,
            step_size: 0.0,
            sample_rate,
            ramp_time_ms,
        }
    }

    pub fn reset(&mut self, value: f32) {
        self.current_value = value;
        self.target_value = value;
        self.step_size = 0.0;
    }

    pub fn set_target(&mut self, target: f32) {
        self.target_value = target;
        let diff = self.target_value - self.current_value;
        
        // Calculate number of steps
        // If sample_rate is actually "calls per second" (e.g. block rate), this logic works.
        // ramp_time_ms / 1000.0 = seconds.
        // seconds * sample_rate = steps.
        
        let steps = (self.ramp_time_ms / 1000.0 * self.sample_rate).max(1.0);
        self.step_size = diff / steps;
    }

    /// 次の値を計算して返す
    pub fn next(&mut self) -> f32 {
        if (self.target_value - self.current_value).abs() > 1e-5 {
            self.current_value += self.step_size;
            // Check overshoot
            if (self.step_size > 0.0 && self.current_value > self.target_value) 
                || (self.step_size < 0.0 && self.current_value < self.target_value) {
                self.current_value = self.target_value;
            }
        } else {
            self.current_value = self.target_value;
        }
        self.current_value
    }
    
    pub fn current(&self) -> f32 {
        self.current_value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smoother_ramp() {
        let sample_rate = 100.0; // 100 blocks per second
        let ramp_time = 50.0; // 50ms = 0.05s
        // Steps = 0.05 * 100 = 5 steps.
        
        let mut smoother = Smoother::new(0.0, sample_rate, ramp_time);
        smoother.set_target(1.0);
        
        // Initial state
        assert_eq!(smoother.current(), 0.0);
        
        // Step 1: 0.2
        assert!((smoother.next() - 0.2).abs() < 1e-5);
        // Step 2: 0.4
        assert!((smoother.next() - 0.4).abs() < 1e-5);
        // Step 3: 0.6
        assert!((smoother.next() - 0.6).abs() < 1e-5);
        // Step 4: 0.8
        assert!((smoother.next() - 0.8).abs() < 1e-5);
        // Step 5: 1.0 (Reached)
        assert!((smoother.next() - 1.0).abs() < 1e-5);
        // Step 6: 1.0 (Clamped)
        assert!((smoother.next() - 1.0).abs() < 1e-5);
    }
}
