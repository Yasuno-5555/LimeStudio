use crate::ir::*;
use crate::graph::AudioGraph;
use crate::compile::{compile_graph, CompiledGraph};
use crate::validate::validate_graph;
use crate::parameter::ParameterRegistry;
use std::sync::Arc;

/// リアルタイム処理を担当する仮想マシンエンジン
pub struct DspEngine {
    pub program: CompiledGraph,
    pub stack: SampleStack,
    
    // Node-to-Node communication buffers
    buffers: Vec<f32>,
    
    // Stateful storage
    delay_lines: Vec<Vec<f32>>,
    delay_pos: Vec<usize>,
    
    // S Tier: Parameter System (S1)
    pub parameters: ParameterRegistry,
    
    // Analysis & Trust UI (Phase 2)
    pub node_peaks: Vec<f32>,
    pub node_rms: Vec<f32>,
    pub scope_buffer: Vec<f32>, 
    pub selected_node_for_scope: Option<crate::graph::NodeId>,
    
    // S Tier: Validation (S3)
    pub hostile_enabled: bool,
    pub error_flags: std::collections::HashSet<String>,
    
    // Phase 2: Trust UI & Modulation
    pub modulation: crate::modulation::ModulationProcessor,
    pub modulated_values: Vec<f32>,
    
    // Phase 3: Visual Compiler (Live Swap)
    pub current_program_version: u64,
    pub pending_program: Option<Arc<CompiledGraph>>,
    
    sample_rate: u32,
}

impl DspEngine {
    pub fn new(graph: &AudioGraph) -> Result<Self, String> {
        let order = validate_graph(graph).map_err(|e| format!("{:?}", e))?;
        let result = compile_graph(graph, &order);
        Ok(Self::new_from_program(result.program))
    }

    pub fn new_from_program(program: CompiledGraph) -> Self {
        let buffers = vec![0.0; program.buffer_count as usize];
        let mut delay_lines = Vec::new();
        let mut delay_pos = Vec::new();
        
        for _ in 0..program.state_count {
            delay_lines.push(vec![0.0; 44100]); // 1s buffer
            delay_pos.push(0);
        }

        Self {
            program,
            stack: SampleStack::new(),
            buffers,
            delay_lines,
            delay_pos,
            parameters: ParameterRegistry::default(),
            node_peaks: vec![0.0; 128],
            node_rms: vec![0.0; 128],
            scope_buffer: vec![0.0; 256],
            selected_node_for_scope: None,
            hostile_enabled: true,
            error_flags: std::collections::HashSet::new(),
            modulation: crate::modulation::ModulationProcessor::new(44100.0),
            modulated_values: vec![0.0; 128],
            current_program_version: 0,
            pending_program: None,
            sample_rate: 44100,
        }
    }
    
    pub fn swap_program(&mut self, program: Arc<CompiledGraph>, version: u64) {
        self.pending_program = Some(program);
        self.current_program_version = version;
    }

    pub fn set_param(&mut self, id: ParamId, value: f32) {
        if let Some(param) = self.parameters.parameters.get(id.0 as usize) {
            param.set_normalized(value);
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
    }

    /// 1サンプルの処理
    pub fn process_sample(&mut self, in_l: f32, in_r: f32) -> (f32, f32) {
        self.stack.clear();

        for op in &self.program.ops {
            match op {
                IrOp::LoadConst(v) => self.stack.push(*v),
                IrOp::LoadParam(id) => {
                    if let Some(param) = self.parameters.parameters.get(id.0 as usize) {
                        let base = param.get_normalized();
                        let offset = self.modulation.get_offset_for(id.0);
                        let final_val = (base + offset).clamp(0.0, 1.0);
                        
                        if (id.0 as usize) < self.modulated_values.len() {
                            self.modulated_values[id.0 as usize] = final_val;
                        }
                        
                        self.stack.push(final_val);
                    } else {
                        self.stack.push(0.0);
                    }
                }
                IrOp::LoadBuffer(id) => {
                    self.stack.push(self.buffers[id.0 as usize]);
                }
                IrOp::StoreBuffer(id) => {
                    self.buffers[id.0 as usize] = self.stack.peek();
                }
                IrOp::Add => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    self.stack.push(a + b);
                }
                IrOp::Mul => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    self.stack.push(a * b);
                }
                IrOp::MulConst(v) => {
                    let a = self.stack.pop();
                    self.stack.push(a * v);
                }
                IrOp::Sub => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    self.stack.push(a - b);
                }
                IrOp::Div => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    if b.abs() < 1e-9 {
                        self.error_flags.insert("Division by zero".into());
                        self.stack.push(0.0);
                    } else {
                        self.stack.push(a / b);
                    }
                }
                IrOp::AddConst(v) => {
                    let a = self.stack.pop();
                    self.stack.push(a + v);
                }
                IrOp::Clamp { min, max } => {
                    let a = self.stack.pop();
                    self.stack.push(a.clamp(*min, *max));
                }
                IrOp::Abs => {
                    let a = self.stack.pop();
                    self.stack.push(a.abs());
                }
                IrOp::Sqrt => {
                    let a = self.stack.pop();
                    self.stack.push(a.sqrt());
                }
                IrOp::Neg => {
                    let a = self.stack.pop();
                    self.stack.push(-a);
                }
                IrOp::Sin => {
                    let a = self.stack.pop();
                    self.stack.push(a.sin());
                }
                IrOp::Cos => {
                    let a = self.stack.pop();
                    self.stack.push(a.cos());
                }
                IrOp::LoadSampleRate => {
                    self.stack.push(self.sample_rate as f32);
                }
                IrOp::Delay { samples, state_id } => {
                    let val = self.stack.pop();
                    let id = state_id.0 as usize;
                    let line = &mut self.delay_lines[id];
                    let pos = &mut self.delay_pos[id];
                    let output = line[*pos];
                    line[*pos] = val;
                    *pos = (*pos + 1) % (*samples as usize).max(1);
                    self.stack.push(output);
                }
                IrOp::ReadInput { channel } => {
                    let val = if *channel == 0 { in_l } else { in_r };
                    self.stack.push(val);
                }
                IrOp::WriteOutput { channel } => {
                    let val = self.stack.pop();
                    let guarded_val = if self.hostile_enabled && (val.is_nan() || val.is_infinite()) {
                        self.error_flags.insert(format!("NaN/Inf in output channel {}", channel));
                        0.0
                    } else {
                        val
                    };
                    
                    if *channel == 0 { self.node_peaks[0] = guarded_val.abs(); }
                    else { self.node_peaks[1] = guarded_val.abs(); }
                    
                    self.stack.push(guarded_val);
                }
                IrOp::CopyBuffer(src, dst) => {
                    self.buffers[dst.0 as usize] = self.buffers[src.0 as usize];
                }
                IrOp::AddBuffer(src, dst) => {
                    self.buffers[dst.0 as usize] += self.buffers[src.0 as usize];
                }
            }
        }

        let out_r = self.stack.pop();
        let out_l = self.stack.pop();

        (out_l, out_r)
    }

    pub fn process_block(&mut self, inputs: &[&[f32]], outputs: &mut [&mut [f32]]) {
        let len = inputs[0].len();
        
        for p in &mut self.node_peaks {
            *p *= 0.95;
        }

        if let Some(new_program) = self.pending_program.take() {
            self.program = (*new_program).clone();
            self.buffers.resize(self.program.buffer_count as usize, 0.0);
        }

        for i in 0..len {
            let in_l = inputs[0][i];
            let in_r = if inputs.len() > 1 { inputs[1][i] } else { in_l };
            
            if i % 64 == 0 {
                self.modulation.process(64);
            }

            let (out_l, out_r) = self.process_sample(in_l, in_r);
            
            outputs[0][i] = out_l;
            if outputs.len() > 1 {
                outputs[1][i] = out_r;
            }

            if let Some(node_id) = self.selected_node_for_scope {
                if node_id.0 < self.buffers.len() {
                    let val = self.buffers[node_id.0];
                    self.scope_buffer.rotate_left(1);
                    if let Some(last) = self.scope_buffer.last_mut() {
                        *last = val;
                    }
                }
            }
        }

        for (i, p) in self.node_peaks.iter().enumerate() {
            if i < self.node_rms.len() {
                self.node_rms[i] = self.node_rms[i] * 0.8 + p * 0.2;
            }
        }
    }
}
