use rtrb::{RingBuffer, Producer, Consumer};
use dirtydata_core::patch::PatchSet;
use dirtydata_core::ir::Graph;
use std::sync::Arc;

/// UI から Engine へ送るコマンド
#[derive(Debug)]
pub enum EngineCommand {
    /// 構造的・パラメータ的な変更の適用
    ApplyPatchSet(PatchSet),
    /// 指定したリビジョンへのチェックアウト（Undo/Redo用）
    Checkout(dirtydata_core::types::PatchId),
}

/// Engine から UI へ返されるレスポンス
#[derive(Debug)]
pub enum EngineResponse {
    /// 変更が確定し、新しいグラフスナップショットが発行された
    SnapshotUpdated {
        snapshot: Arc<Graph>,
        trace_id: Option<ulid::Ulid>,
    },
    /// エラー報告
    Error {
        message: String,
        trace_id: Option<ulid::Ulid>,
    },
    /// テレメトリ・因果関係データ
    Telemetry {
        cpu_load: f32,
        causality_events: Vec<crate::causality::PolyphonicCausality>,
        node_cpu: std::collections::HashMap<dirtydata_core::StableId, f32>,
    },
}

pub struct UiToEnginePipeline {
    producer: Producer<EngineCommand>,
}

pub struct EngineToUiPipeline {
    consumer: Consumer<EngineResponse>,
}

pub struct PipelineFactory;

impl PipelineFactory {
    pub fn create_pair(capacity: usize) -> (UiToEnginePipeline, EngineToUiPipeline, Producer<EngineResponse>, Consumer<EngineCommand>) {
        let (cmd_p, cmd_c) = RingBuffer::new(capacity);
        let (res_p, res_c) = RingBuffer::new(capacity);
        
        (
            UiToEnginePipeline { producer: cmd_p },
            EngineToUiPipeline { consumer: res_c },
            res_p,
            cmd_c,
        )
    }
}

impl UiToEnginePipeline {
    pub fn send(&mut self, cmd: EngineCommand) -> Result<(), rtrb::PushError<EngineCommand>> {
        self.producer.push(cmd)
    }
}

impl EngineToUiPipeline {
    pub fn try_recv(&mut self) -> Option<EngineResponse> {
        self.consumer.pop().ok()
    }
}
