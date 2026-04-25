//! Compiler Lens & Inspector — Visible Intelligence.
//! 
//! Real-time visualization of the compilation process and validation.

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerLens {
    pub node_id: u32,
    pub ir_instructions: Vec<String>,
    pub rust_equivalent: String,
    pub validation_status: ValidationState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationState {
    Clean,
    Warning(String),
    Error(String),
    Hostile(Vec<String>),
}

pub struct InspectorState {
    pub active_lens: Option<CompilerLens>,
    pub show_ir: bool,
    pub show_rust: bool,
    pub show_provenance: bool,
}

impl InspectorState {
    pub fn new() -> Self {
        Self {
            active_lens: None,
            show_ir: true,
            show_rust: true,
            show_provenance: true,
        }
    }
}
