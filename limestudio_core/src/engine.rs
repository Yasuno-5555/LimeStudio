use crate::ir::{IrOp, SampleStack, ParamId};
use crate::compile::{CompiledGraph, compile_graph};
use crate::graph::AudioGraph;
use crate::validate::validate_graph;

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

pub struct DspEngine {
    program: CompiledGraph,
    stack: SampleStack,
    buffers: Vec<f32>,
    
    // Stateful storage
    delay_lines: Vec<Vec<f32>>,
    delay_pos: Vec<usize>,
    
    // Civilized Atomics: AtomicU32 for f32 values
    param_values: Vec<AtomicU32>,
    
    sample_rate: u32,
}

impl DspEngine {
    pub fn new(graph: &AudioGraph) -> Result<Self, String> {
        let order = validate_graph(graph).map_err(|e| format!("{:?}", e))?;
        let program = compile_graph(graph, &order);
        Ok(Self::new_from_program(program))
    }

    pub fn new_from_program(program: CompiledGraph) -> Self {
        let buffers = vec![0.0; program.buffer_count as usize];
        let mut delay_lines = Vec::new();
        let mut delay_pos = Vec::new();
        
        let mut param_values = Vec::new();
        for _ in 0..128 {
            param_values.push(AtomicU32::new(0.0f32.to_bits()));
        }

        for _ in 0..program.state_count {
            delay_lines.push(vec![0.0; 44100]); // 1秒分
            delay_pos.push(0);
        }

        Self {
            program,
            stack: SampleStack::new(),
            buffers,
            delay_lines,
            delay_pos,
            param_values,
            sample_rate: 44100, // Default
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
    }

    /// パラメータ値を更新 (Atomic)
    pub fn set_param(&self, id: ParamId, value: f32) {
        if (id.0 as usize) < self.param_values.len() {
            self.param_values[id.0 as usize].store(value.to_bits(), Ordering::Relaxed);
        }
    }

    /// 1サンプル処理 (Main Loop)
    #[inline]
    pub fn process_sample(&mut self, input_l: f32, input_r: f32) -> (f32, f32) {
        self.stack.clear();
        let mut out_l = 0.0;
        let mut out_r = 0.0;

        for op in &self.program.ops {
            match op {
                IrOp::LoadConst(v) => self.stack.push(*v),
                IrOp::LoadParam(id) => {
                    let bits = self.param_values[id.0 as usize].load(Ordering::Relaxed);
                    self.stack.push(f32::from_bits(bits));
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
                IrOp::AddConst(v) => {
                    let a = self.stack.pop();
                    self.stack.push(a + v);
                }
                IrOp::Sub => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    self.stack.push(a - b);
                }
                IrOp::Div => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    if b != 0.0 {
                        self.stack.push(a / b);
                    } else {
                        self.stack.push(0.0);
                    }
                }
                IrOp::ReadInput { channel } => {
                    if *channel == 0 {
                        self.stack.push(input_l);
                    } else if *channel == 1 {
                        self.stack.push(input_r);
                    } else {
                        self.stack.push(0.0);
                    }
                }
                IrOp::WriteOutput { channel } => {
                    let val = self.stack.pop();
                    if *channel == 0 {
                        out_l = val;
                    } else if *channel == 1 {
                        out_r = val;
                    }
                }
                IrOp::CopyBuffer(src, dst) => {
                    self.buffers[dst.0 as usize] = self.buffers[src.0 as usize];
                }
                IrOp::AddBuffer(src, dst) => {
                    self.buffers[dst.0 as usize] += self.buffers[src.0 as usize];
                }
                IrOp::Clamp { min, max } => {
                    let v = self.stack.pop();
                    self.stack.push(v.clamp(*min, *max));
                }
                IrOp::Abs => {
                    let v = self.stack.pop();
                    self.stack.push(v.abs());
                }
                IrOp::Sqrt => {
                    let v = self.stack.pop();
                    self.stack.push(v.sqrt());
                }
                IrOp::Neg => {
                    let v = self.stack.pop();
                    self.stack.push(-v);
                }
                IrOp::Sin => {
                    let v = self.stack.pop();
                    self.stack.push(v.sin());
                }
                IrOp::Cos => {
                    let v = self.stack.pop();
                    self.stack.push(v.cos());
                }
                IrOp::LoadSampleRate => {
                    self.stack.push(self.sample_rate as f32);
                }
                IrOp::Delay { samples, state_id } => {
                    let input = self.stack.pop();
                    let id = state_id.0 as usize;
                    let line = &mut self.delay_lines[id];
                    let pos = &mut self.delay_pos[id];
                    
                    let output = line[*pos];
                    line[*pos] = input;
                    
                    *pos = (*pos + 1) % (*samples as usize).max(1);
                    self.stack.push(output);
                }
            }
        }

        (out_l, out_r)
    }

    /// ブロック処理
    pub fn process_block(&mut self, inputs: &[&[f32]], outputs: &mut [&mut [f32]]) {
        let len = inputs[0].len();
        for i in 0..len {
            let in_l = inputs[0][i];
            let in_r = if inputs.len() > 1 { inputs[1][i] } else { in_l };
            
            let (out_l, out_r) = self.process_sample(in_l, in_r);
            
            outputs[0][i] = out_l;
            if outputs.len() > 1 {
                outputs[1][i] = out_r;
            }
        }
    }
}
