pub mod builder;
pub mod causality;
pub mod diff;
pub mod engine;
#[cfg(test)]
mod engine_tests;
pub mod graph;
pub mod math;
pub mod node_discovery;
pub mod pipeline;
pub mod preset;
pub mod project;
pub mod provenance;
pub mod reality_bridge_validation;
pub mod signal;
pub mod telemetry;
pub mod time;
pub mod topology;
pub mod torture;
pub mod transaction;
pub mod view;

pub use project::ProjectSpec;
pub use view::{UiIndex, ViewCache};

/// 汎用的なオーディオバッファインターフェース
pub trait AudioBuffer {
    fn channels(&self) -> usize;
    fn samples(&self) -> usize;
    fn get_channel(&self, ch: usize) -> &[f32];
    fn get_channel_mut(&mut self, ch: usize) -> &mut [f32];
}

/// 処理コンテキスト
pub struct ProcessContext {
    pub sample_rate: f64,
    pub max_block_size: usize,
    pub num_channels: usize,
}

impl ProcessContext {
    pub fn new(sample_rate: f64, max_block_size: usize, num_channels: usize) -> Self {
        Self {
            sample_rate,
            max_block_size,
            num_channels,
        }
    }
}

/// 基本的なオーディオプロセッサートレイト
pub trait AudioProcessor: Send + Sync {
    fn prepare(&mut self, context: &ProcessContext);
    fn process<B: AudioBuffer>(&mut self, buffer: &mut B);
    fn latency(&self) -> u32 {
        0
    }
    fn set_parameter(&mut self, _id: u32, _value: f32) {}
}

#[derive(Debug, Clone)]
pub enum Intent {
    AddNode {
        kind: String,
        position: [f32; 2],
    },
    RemoveNode(UiIndex),
    MoveNode {
        node_id: UiIndex,
        position: [f32; 2],
    },
    Connect {
        from: UiIndex,
        to: UiIndex,
    },
    TweakParam {
        node_id: UiIndex,
        param: String,
        value: f32,
    },
    EndTweak {
        node_id: UiIndex,
        param: String,
    },
    SelectNodes(Vec<UiIndex>),
}

/// リアルタイムオーディオスレッド向けの軽量なパラメータ更新イベント
#[derive(Debug, Clone)]
pub enum PatchEvent {
    SetParameter {
        param_id: String, // NIH-plug parameter ID
        value: f32,
    },
}
