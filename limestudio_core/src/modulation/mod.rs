//! Modulation Engine (Phase 2: Trust UI Foundations)
//! 
//! リアルタイムでのLFO、エンベロープ、およびルーティングの計算を担当。
//! 「音がどう変わっているか」をUIにフィードバックするためのデータの源泉。

use crate::param_graph::{ModSource, ModRouting, LfoShape, ProcessRate};
use std::collections::HashMap;

/// リアルタイム・モジュレーション・プロセッサ
pub struct ModulationProcessor {
    pub sources: Vec<ModSourceInstance>,
    pub routings: Vec<ModRouting>,
    /// パラメータごとの現在のモジュレーションオフセット値
    /// (ターゲットパラメータID -> 合計オフセット)
    pub offsets: HashMap<u32, f32>,
    
    sample_rate: f64,
}

/// 実行状態を持つモジュレーションソース
pub struct ModSourceInstance {
    pub definition: ModSource,
    pub phase: f32,
    /// 現在の出力値 (-1.0 to 1.0 or 0.0 to 1.0)
    pub current_output: f32,
}

impl ModulationProcessor {
    pub fn new(sample_rate: f64) -> Self {
        Self {
            sources: Vec::new(),
            routings: Vec::new(),
            offsets: HashMap::new(),
            sample_rate,
        }
    }

    pub fn add_source(&mut self, source: ModSource) {
        self.sources.push(ModSourceInstance {
            definition: source,
            phase: 0.0,
            current_output: 0.0,
        });
    }

    pub fn add_routing(&mut self, routing: ModRouting) {
        self.routings.push(routing);
    }

    /// 1サンプル（または1ブロック）進める
    pub fn process(&mut self, samples: u32) {
        // 1. 各ソースの状態更新
        for src in &mut self.sources {
            match &src.definition.kind {
                crate::param_graph::ModSourceKind::Lfo { shape, sync } => {
                    let hz = match sync {
                        crate::param_graph::TempoSync::Free { hz } => *hz,
                        _ => 1.0, // TODO: Tempo sync
                    };
                    
                    let phase_inc = (hz as f64 / self.sample_rate) as f32;
                    src.phase = (src.phase + phase_inc * samples as f32) % 1.0;
                    
                    src.current_output = match shape {
                        LfoShape::Sine => (src.phase * std::f32::consts::TAU).sin(),
                        LfoShape::Triangle => (src.phase * 2.0 - 1.0).abs() * 2.0 - 1.0,
                        LfoShape::Saw => src.phase * 2.0 - 1.0,
                        LfoShape::Square => if src.phase < 0.5 { 1.0 } else { -1.0 },
                        LfoShape::SampleAndHold => src.current_output, // TODO: Implementation
                    };
                }
                _ => {}
            }
        }

        // 2. ルーティングに基づいてオフセットを計算
        self.offsets.clear();
        for routing in &self.routings {
            if let Some(src) = self.sources.iter().find(|s| s.definition.id == routing.source_id) {
                let amount = routing.amount;
                let val = src.current_output * amount;
                
                let entry = self.offsets.entry(routing.target_param.0).or_insert(0.0);
                *entry += val;
            }
        }
    }

    /// 特定のパラメータに適用される総モジュレーション量を取得
    pub fn get_offset_for(&self, param_id: u32) -> f32 {
        *self.offsets.get(&param_id).unwrap_or(&0.0)
    }
}

/// UIへのフィードバック用データ (Trust UI)
#[derive(Clone, Debug)]
pub struct ModulationFeedback {
    pub source_id: u32,
    pub current_value: f32,
}
