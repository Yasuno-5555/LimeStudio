use crate::engine::DspEngine;
use crate::compile::CompiledGraph;
use crate::ir::{ParamRef, ParamId, BufferId};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VoiceState {
    Idle,
    Active,
    Releasing,
}

pub struct Voice {
    pub id: u32,
    pub note: u8,
    pub velocity: f32,
    pub state: VoiceState,
    pub age: u64, // カウントアップして stealing に使用
    pub engine: DspEngine,
}

pub struct VoiceAllocator {
    pub voices: Vec<Voice>,
    pub max_voices: usize,
    pub current_age: u64,
    
    // パラメータのマッピング用 (簡易版)
    pub freq_param_id: Option<ParamId>,
    pub gate_param_id: Option<ParamId>,
    pub vel_param_id: Option<ParamId>,
}

impl VoiceAllocator {
    pub fn new(program: &CompiledGraph, max_voices: usize) -> Self {
        let mut voices = Vec::with_capacity(max_voices);
        for i in 0..max_voices {
            voices.push(Voice {
                id: i as u32,
                note: 0,
                velocity: 0.0,
                state: VoiceState::Idle,
                age: 0,
                engine: DspEngine::new_from_program(program.clone()),
            });
        }
        
        Self {
            voices,
            max_voices,
            current_age: 0,
            freq_param_id: None,
            gate_param_id: None,
            vel_param_id: None,
        }
    }

    pub fn set_parameter_mapping(&mut self, freq: Option<ParamId>, gate: Option<ParamId>, vel: Option<ParamId>) {
        self.freq_param_id = freq;
        self.gate_param_id = gate;
        self.vel_param_id = vel;
    }

    pub fn note_on(&mut self, note: u8, velocity: f32) {
        self.current_age += 1;
        let age = self.current_age;
        
        // 1. すでに同じノートが鳴っているか確認 (Re-trigger)
        for voice in &mut self.voices {
            if voice.state != VoiceState::Idle && voice.note == note {
                Self::init_voice_internal(voice, note, velocity, age, self.freq_param_id, self.gate_param_id, self.vel_param_id);
                return;
            }
        }
        
        // 2. 空いているボイスを探す
        for voice in &mut self.voices {
            if voice.state == VoiceState::Idle {
                Self::init_voice_internal(voice, note, velocity, age, self.freq_param_id, self.gate_param_id, self.vel_param_id);
                return;
            }
        }
        
        // 3. ボイスを盗む (Oldest stealing)
        let mut oldest_idx = 0;
        let mut oldest_age = u64::MAX;
        for (i, voice) in self.voices.iter().enumerate() {
            if voice.age < oldest_age {
                oldest_age = voice.age;
                oldest_idx = i;
            }
        }
        
        let voice = &mut self.voices[oldest_idx];
        Self::init_voice_internal(voice, note, velocity, age, self.freq_param_id, self.gate_param_id, self.vel_param_id);
    }

    fn init_voice_internal(
        voice: &mut Voice, 
        note: u8, 
        velocity: f32, 
        age: u64,
        freq_id: Option<ParamId>,
        gate_id: Option<ParamId>,
        vel_id: Option<ParamId>
    ) {
        voice.note = note;
        voice.velocity = velocity;
        voice.state = VoiceState::Active;
        voice.age = age;
        
        // パラメータ更新
        if let Some(id) = freq_id {
            let freq = 440.0 * 2.0f32.powf((note as f32 - 69.0) / 12.0);
            voice.engine.set_param(id, freq);
        }
        if let Some(id) = gate_id {
            voice.engine.set_param(id, 1.0);
        }
        if let Some(id) = vel_id {
            voice.engine.set_param(id, velocity);
        }
    }

    pub fn note_off(&mut self, note: u8) {
        for voice in &mut self.voices {
            if voice.note == note && voice.state == VoiceState::Active {
                voice.state = VoiceState::Releasing;
                if let Some(id) = self.gate_param_id {
                    voice.engine.set_param(id, 0.0);
                }
                
                // TODO: 本来は ADSR が完了するまで Idle にしないが、
                // 今回はダミー ADSR なので即座に Idle にする設定も可能
                // 一旦、簡易的に Idle に戻す
                voice.state = VoiceState::Idle;
            }
        }
    }

    pub fn process_sample(&mut self, input_l: f32, input_r: f32) -> (f32, f32) {
        let mut sum_l = 0.0;
        let mut sum_r = 0.0;
        
        for voice in &mut self.voices {
            if voice.state != VoiceState::Idle {
                let (v_l, v_r) = voice.engine.process_sample(input_l, input_r);
                sum_l += v_l;
                sum_r += v_r;
            }
        }
        
        (sum_l, sum_r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::*;
    use crate::stdlib::*;
    use crate::compile::*;
    use crate::ir::ParamRef;

    fn create_test_program() -> CompiledGraph {
        let mut g = AudioGraph::new();
        g.add_node(GraphNode::Input { channel: 0 }); // Node 0
        g.add_node(GraphNode::Stdlib(StdlibNode::Oscillator { // Node 1
            freq: ParamRef::Param(ParamId(0)),
            wave: Waveform::Sine,
        }));
        g.add_node(GraphNode::Output { channel: 0 }); // Node 2
        g.add_edge(NodeId(1), 0, NodeId(2), 0);
        
        let order = crate::validate::validate_graph(&g).unwrap();
        compile_graph(&g, &order)
    }

    #[test]
    fn test_voice_allocation() {
        let prog = create_test_program();
        let mut allocator = VoiceAllocator::new(&prog, 2);
        allocator.set_parameter_mapping(Some(ParamId(0)), Some(ParamId(1)), None);

        // 1st note
        allocator.note_on(60, 0.8);
        assert_eq!(allocator.voices[0].note, 60);
        assert_eq!(allocator.voices[0].state, VoiceState::Active);

        // 2nd note
        allocator.note_on(64, 0.7);
        assert_eq!(allocator.voices[1].note, 64);
        assert_eq!(allocator.voices[1].state, VoiceState::Active);

        // 3rd note (steal oldest, which is voice 0)
        allocator.note_on(67, 0.9);
        assert_eq!(allocator.voices[0].note, 67);
        assert_eq!(allocator.voices[0].state, VoiceState::Active);
        
        // Note off
        allocator.note_off(64);
        assert_eq!(allocator.voices[1].state, VoiceState::Idle);
    }
}
