//! Patch Provenance (Tier Ω)
//! 
//! 「なぜ今この値なのか」を見せる。
//! Macro -> LFO -> Envelope -> Host Automation の連鎖を記録。

use serde::{Serialize, Deserialize};
use crate::ir::ParamId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterProvenance {
    pub param_id: ParamId,
    pub base_value: f32,
    pub modifiers: Vec<ModifierEffect>,
    pub final_value: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifierEffect {
    pub source_name: String,
    pub amount: f32,
    pub contribution: f32, // Final effect on the parameter
}

impl ParameterProvenance {
    pub fn new(param_id: ParamId, base_value: f32) -> Self {
        Self {
            param_id,
            base_value,
            modifiers: Vec::new(),
            final_value: base_value,
        }
    }

    pub fn add_modifier(&mut self, source: &str, amount: f32, contribution: f32) {
        self.modifiers.push(ModifierEffect {
            source_name: source.to_string(),
            amount,
            contribution,
        });
        self.final_value += contribution;
    }

    pub fn print_trace(&self) {
        println!("Provenance for P{}:", self.param_id.0);
        println!("  Base: {:.4}", self.base_value);
        for m in &self.modifiers {
            println!("  [+] {} (amt: {:.2}) -> contribution: {:.4}", 
                m.source_name, m.amount, m.contribution);
        }
        println!("  Final: {:.4}", self.final_value);
    }
}
