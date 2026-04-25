use rhai::{Engine, Scope, Dynamic};
use crate::ir::{IrOp, ParamRef, BufferId};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug)]
pub struct Signal(pub BufferId);

pub struct ScriptContext {
    pub ops: Vec<IrOp>,
    pub next_buffer: usize,
    pub input_signals: Vec<Signal>,
    pub output_signals: Vec<Signal>,
}

impl ScriptContext {
    pub fn new(inputs: Vec<BufferId>, outputs: Vec<BufferId>, start_temp_buffer: usize) -> Self {
        Self {
            ops: Vec::new(),
            next_buffer: start_temp_buffer,
            input_signals: inputs.into_iter().map(Signal).collect(),
            output_signals: outputs.into_iter().map(Signal).collect(),
        }
    }

    pub fn alloc_buffer(&mut self) -> BufferId {
        let id = BufferId(self.next_buffer as u32);
        self.next_buffer += 1;
        id
    }
}

pub fn run_script(source: &str, inputs: Vec<BufferId>, outputs: Vec<BufferId>, start_temp: usize) -> Result<(Vec<IrOp>, usize), String> {
    let mut engine = Engine::new();
    
    // The state must be shared with Rhai functions
    let ctx = Arc::new(Mutex::new(ScriptContext::new(inputs, outputs, start_temp)));

    // API: input(index) -> Signal
    let c = ctx.clone();
    engine.register_fn("input", move |idx: i64| -> Result<Signal, Box<rhai::EvalAltResult>> {
        let ctx = c.lock().unwrap();
        ctx.input_signals.get(idx as usize).cloned().ok_or_else(|| Box::<rhai::EvalAltResult>::from("Input index out of bounds"))
    });

    // API: mul(Signal, FLOAT) -> Signal
    let c = ctx.clone();
    engine.register_fn("mul", move |sig: Signal, factor: rhai::FLOAT| -> Signal {
        let mut ctx = c.lock().unwrap();
        let new_buf = ctx.alloc_buffer();
        ctx.ops.push(IrOp::LoadBuffer(sig.0));
        ctx.ops.push(IrOp::MulConst(factor as f32));
        ctx.ops.push(IrOp::StoreBuffer(new_buf));
        Signal(new_buf)
    });
    
    let c = ctx.clone();
    engine.register_fn("add", move |sig1: Signal, sig2: Signal| -> Signal {
        let mut ctx = c.lock().unwrap();
        let new_buf = ctx.alloc_buffer();
        ctx.ops.push(IrOp::LoadBuffer(sig1.0));
        ctx.ops.push(IrOp::LoadBuffer(sig2.0));
        ctx.ops.push(IrOp::Add);
        ctx.ops.push(IrOp::StoreBuffer(new_buf));
        Signal(new_buf)
    });

    // API: output(index, Signal)
    let c = ctx.clone();
    engine.register_fn("output", move |idx: i64, sig: Signal| -> Result<(), Box<rhai::EvalAltResult>> {
        let mut ctx = c.lock().unwrap();
        let out_sig = ctx.output_signals.get(idx as usize).cloned().ok_or_else(|| Box::<rhai::EvalAltResult>::from("Output index out of bounds"))?;
        ctx.ops.push(IrOp::CopyBuffer(sig.0, out_sig.0));
        Ok(())
    });

    // Rhai type registration
    engine.register_type_with_name::<Signal>("Signal");

    let mut scope = Scope::new();
    engine.run_with_scope(&mut scope, source).map_err(|e| e.to_string())?;

    let res = ctx.lock().unwrap();
    Ok((res.ops.clone(), res.next_buffer))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_gain_ir_generation() {
        let source = r#"
            let x = input(0);
            let y = mul(x, 0.5);
            output(0, y);
        "#;
        
        let inputs = vec![BufferId(0)];
        let outputs = vec![BufferId(1)];
        let (ops, _) = run_script(source, inputs, outputs, 10).unwrap();
        
        // Expected IR:
        // LoadBuffer(0)
        // MulConst(0.5)
        // StoreBuffer(10)
        // CopyBuffer(10, 1)
        
        assert_eq!(ops.len(), 4);
        match &ops[0] {
            IrOp::LoadBuffer(buf) => assert_eq!(buf.0, 0),
            _ => panic!("Expected LoadBuffer"),
        }
        match &ops[1] {
            IrOp::MulConst(val) => assert_eq!(*val, 0.5),
            _ => panic!("Expected MulConst"),
        }
        match &ops[2] {
            IrOp::StoreBuffer(buf) => assert_eq!(buf.0, 10),
            _ => panic!("Expected StoreBuffer"),
        }
        match &ops[3] {
            IrOp::CopyBuffer(src, dst) => {
                assert_eq!(src.0, 10);
                assert_eq!(dst.0, 1);
            },
            _ => panic!("Expected CopyBuffer"),
        }
    }
}
