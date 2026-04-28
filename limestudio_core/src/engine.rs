use crate::PatchEvent;
use dirtydata_runtime::jit::{JitProgram, JitCompiler};
use dirtydata_runtime::nodes::base::ProcessContext as DspProcessContext;
use rtrb::Consumer;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VoiceState {
    Idle,
    Active { pitch: u8, velocity: f32, age: u64 },
    Release { pitch: u8, age: u64 },
    Stealing { 
        pitch: u8, 
        age: u64, 
        fade_out_samples: u32,
        next_note: Option<(u8, f32)>,
    },
}

pub enum VoiceEvent {
    NoteOn { pitch: u8, velocity: f32 },
    NoteOff { pitch: u8 },
    Pressure { pitch: u8, value: f32 },
    Tuning { pitch: u8, value: f32 },
}

pub struct VoicePlan {
    pub plan: JitProgram,
    pub state: VoiceState,
    pub last_level: f32,
}

impl VoicePlan {
    pub fn new(plan: JitProgram) -> Self {
        Self {
            plan,
            state: VoiceState::Idle,
            last_level: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.state = VoiceState::Idle;
        self.last_level = 0.0;
        self.plan.reset();
    }

    pub fn score(&self) -> f32 {
        match self.state {
            VoiceState::Idle => f32::INFINITY,
            VoiceState::Active { age, .. } => {
                // High importance, but older voices are slightly more stealable
                1000.0 + self.last_level * 100.0 - (age as f32 * 0.00001)
            }
            VoiceState::Release { age, .. } => {
                // Lower importance
                500.0 + self.last_level * 50.0 - (age as f32 * 0.0001)
            }
            VoiceState::Stealing { .. } => -1.0, // Already being stolen
        }
    }

    pub fn process_sample(&mut self, ctx: &DspProcessContext) -> [f32; 2] {
        let mut out = self.plan.execute(ctx);
        
        match &mut self.state {
            VoiceState::Active { age, .. } | VoiceState::Release { age, .. } => {
                *age += 1;
            }
            VoiceState::Stealing { fade_out_samples, next_note, .. } => {
                let gain = *fade_out_samples as f32 / 220.0;
                out[0] *= gain;
                out[1] *= gain;
                if *fade_out_samples > 0 {
                    *fade_out_samples -= 1;
                } else if let Some((new_pitch, new_velocity)) = *next_note {
                    self.state = VoiceState::Active { pitch: new_pitch, velocity: new_velocity, age: 0 };
                    self.state = VoiceState::Active { pitch: new_pitch, velocity: new_velocity, age: 0 };
                    self.plan.set_parameter_by_name("gate", 1.0);
                    self.plan.set_parameter_by_name("velocity", new_velocity);
                    self.plan.set_parameter_by_name("pitch", new_pitch as f32);
                } else {
                    self.state = VoiceState::Idle;
                }
            }
            VoiceState::Idle => {
                out = [0.0, 0.0];
            }
        }
        
        self.last_level = (out[0].abs() + out[1].abs()) * 0.5;
        out
    }
}

pub struct VoiceManager {
    pub voices: Vec<VoicePlan>,
    pub patch_rx: Consumer<PatchEvent>,
    pub sample_rate: f32,
    pub signal_registry: Option<std::sync::Arc<crate::signal::SignalRegistry>>,
    pub telemetry: Option<crate::telemetry::TelemetryProducer>,
}

impl VoiceManager {
    pub fn new(
        voices: Vec<VoicePlan>, 
        patch_rx: Consumer<PatchEvent>, 
        sample_rate: f32,
        telemetry: Option<crate::telemetry::TelemetryProducer>,
    ) -> Self {
        Self {
            voices,
            patch_rx,
            sample_rate,
            signal_registry: None,
            telemetry,
        }
    }

    pub fn from_graph(
        graph: &dirtydata_core::ir::Graph, 
        patch_rx: Consumer<PatchEvent>, 
        num_voices: usize, 
        sample_rate: f32,
        telemetry: Option<crate::telemetry::TelemetryProducer>,
    ) -> Self {
        let dsp_runner = dirtydata_runtime::DspRunner::new(graph.clone(), None, sample_rate);
        
        let mut voices = Vec::with_capacity(num_voices);
        for _ in 0..num_voices {
            let mut compiler = JitCompiler::new();
            let plan = compiler.compile_runner(&dsp_runner).expect("Failed to compile JIT program");
            voices.push(VoicePlan::new(plan));
        }
        
        Self::new(voices, patch_rx, sample_rate, telemetry)
    }

    pub fn reset(&mut self) {
        for voice in &mut self.voices {
            voice.reset();
        }
    }

    pub fn handle_event(&mut self, event: VoiceEvent) {
        match event {
            VoiceEvent::NoteOn { pitch, velocity } => {
                // 1. Find an idle voice
                if let Some((idx, voice)) = self.voices.iter_mut().enumerate().find(|(_, v)| v.state == VoiceState::Idle) {
                    voice.state = VoiceState::Active { pitch, velocity, age:0 };
                    voice.plan.set_parameter_by_name("gate", 1.0);
                    voice.plan.set_parameter_by_name("velocity", velocity);
                    voice.plan.set_parameter_by_name("pitch", pitch as f32);
                    voice.plan.set_parameter_by_name("tuning", 0.0); // Reset tuning for new note
                    
                    if let Some(tel) = &mut self.telemetry {
                        tel.push(crate::telemetry::TelemetryEvent::VoiceActive { index: idx, pitch, velocity });
                    }
                    return;
                }
                
                // 2. No idle voice, find best candidate to steal
                let mut best_idx = 0;
                let mut min_score = f32::INFINITY;
                for (i, voice) in self.voices.iter().enumerate() {
                    let s = voice.score();
                    if s < min_score {
                        min_score = s;
                        best_idx = i;
                    }
                }
                
                let voice = &mut self.voices[best_idx];
                match voice.state {
                    VoiceState::Active { pitch: old_pitch, age, .. } | VoiceState::Release { pitch: old_pitch, age } => {
                        voice.state = VoiceState::Stealing { 
                            pitch: old_pitch, 
                            age, 
                            fade_out_samples: 220,
                            next_note: Some((pitch, velocity)),
                        };
                        voice.plan.set_parameter_by_name("gate", 0.0);
                        voice.plan.set_parameter_by_name("tuning", 0.0); // Reset tuning
                    }
                    VoiceState::Stealing { ref mut next_note, .. } => {
                        // If already stealing, just replace the next note
                        *next_note = Some((pitch, velocity));
                    }
                    _ => {}
                }
                
                // Note: The new note is lost or we'd need a "pending" queue.
                // For this implementation, we'll just start the new note in the NEXT available slot.
            }
            VoiceEvent::NoteOff { pitch } => {
                for (idx, voice) in self.voices.iter_mut().enumerate() {
                    if let VoiceState::Active { pitch: p, age, .. } = voice.state {
                        if p == pitch {
                            voice.state = VoiceState::Release { pitch, age };
                            voice.plan.set_parameter_by_name("gate", 0.0);
                            
                            if let Some(tel) = &mut self.telemetry {
                                tel.push(crate::telemetry::TelemetryEvent::VoiceReleased { index: idx });
                            }
                        }
                    }
                }
            }
            VoiceEvent::Pressure { pitch, value } => {
                for voice in self.voices.iter_mut() {
                    if let VoiceState::Active { pitch: p, .. } = voice.state {
                        if p == pitch {
                            voice.plan.set_parameter_by_name("pressure", value);
                        }
                    }
                }
            }
            VoiceEvent::Tuning { pitch, value } => {
                for voice in self.voices.iter_mut() {
                    if let VoiceState::Active { pitch: p, .. } = voice.state {
                        if p == pitch {
                            voice.plan.set_parameter_by_name("tuning", value);
                        }
                    }
                }
            }
        }
    }

    pub fn process(&mut self, inputs: &[&[f32]], outputs: &mut [&mut [f32]], sample_rate: f32) {
        self.sample_rate = sample_rate;
        let block_start = std::time::Instant::now();
        
        // 1. Draining parameter updates
        while let Ok(event) = self.patch_rx.pop() {
            match event {
                PatchEvent::SetParameter { param_id, value } => {
                    for voice in &mut self.voices {
                        voice.plan.set_parameter_by_name(&param_id, value);
                    }
                }
            }
        }

        if outputs.is_empty() { return; }
        let samples = outputs[0].len();
        let num_outputs = outputs.len();

        // Zero out outputs first
        for output in outputs.iter_mut().take(num_outputs) {
            for val in output.iter_mut().take(samples) {
                *val = 0.0;
            }
        }

        let mut peak_activity = 0.0f32;

        for i in 0..samples {
            let ctx = DspProcessContext {
                sample_rate,
                global_sample_index: i as u64,
                crash_flag: None,
                osc_tx: None,
                convergence_info: None,
                node_diagnostics: None,
                node_id: None,
            };

            for voice in &mut self.voices {
                if voice.state == VoiceState::Idle { continue; }
                
                // Setup input registers (Shared for all voices)
                if !inputs.is_empty() {
                    let _input_l = inputs[0][i];
                    let _input_r = if inputs.len() > 1 { inputs[1][i] } else { _input_l };
                    // TODO: Map inputs to JIT memory if needed
                }

                let out = voice.process_sample(&ctx);
                
                // NaN Detection
                if out[0].is_nan() || out[1].is_nan() {
                    if let Some(tel) = &mut self.telemetry {
                        tel.push(crate::telemetry::TelemetryEvent::NanDetected { node_id: None });
                    }
                }

                outputs[0][i] += out[0];
                if num_outputs > 1 {
                    outputs[1][i] += out[1];
                }
                
                peak_activity = peak_activity.max(out[0].abs()).max(out[1].abs());
            }
        }

        // Report metrics to the Truth Registry and Telemetry Bridge
        let elapsed_us = block_start.elapsed().as_micros() as f32;
        if let Some(reg) = &self.signal_registry {
            reg.set_metric("engine/cpu_us", elapsed_us);
            reg.set_metric("engine/activity", peak_activity);
        }
        if let Some(tel) = &mut self.telemetry {
            tel.push(crate::telemetry::TelemetryEvent::CpuUsage { micros: elapsed_us });
            if peak_activity > 0.99 {
                tel.push(crate::telemetry::TelemetryEvent::ClipDetected { channel: 0, peak: peak_activity });
            }
        }
    }

    pub fn latency_samples(&self) -> u32 {
        self.voices.first().map(|v| v.plan.latency_samples()).unwrap_or(0)
    }
}
