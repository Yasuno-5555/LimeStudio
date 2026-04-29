use crate::pipeline::{EngineCommand, EngineResponse, EngineToUiPipeline, UiToEnginePipeline};
use crate::{Intent, ViewCache};
use dirtydata_core::{ConfigChange, ConfigValue, StableId};
use dirtydata_core::{Operation, Patch, PatchSet};
use std::collections::{BTreeMap, HashMap, VecDeque};
use tracing::{error, info, warn};

/// 楽観的更新の状態を追跡するためのレコード
/// The source of a change, defining responsibility.
#[derive(Debug, Clone, PartialEq)]
pub enum Author {
    User,
    HostAutomation(String), // Host name or param ID
    Script(String),         // Script ID
    System,
}

impl std::fmt::Display for Author {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Author::User => write!(f, "User"),
            Author::HostAutomation(h) => write!(f, "Host: {}", h),
            Author::Script(s) => write!(f, "Script: {}", s),
            Author::System => write!(f, "System"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransactionMetadata {
    pub author: Author,
    pub timestamp: crate::time::Timestamp,
    pub intent: Intent,
}

struct PendingTransaction {
    trace_id: ulid::Ulid,
    metadata: TransactionMetadata,
    _patch_set: Option<PatchSet>,
}

/// Transaction Layer - "Semantic Compiler + State Guard"
pub struct HistoryNode {
    pub patch_id: dirtydata_core::types::PatchId,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    pub project: crate::project::ProjectSpec,
    pub metadata: TransactionMetadata,
}

pub struct TransactionLayer {
    /// The Single Source of Truth
    project: crate::project::ProjectSpec,

    /// 継続的なパラメータ変更のバッファ
    _ongoing_tweaks: HashMap<(dirtydata_core::StableId, String), f32>,

    /// 承認待ちのトランザクションキュー（Reconciliation用）
    pending_queue: VecDeque<PendingTransaction>,

    /// 履歴ツリー（不変の歴史）
    pub history_nodes: Vec<HistoryNode>,
    pub current_history_idx: Option<usize>,

    /// 監査ログ (将来的に history_nodes に統合可能だが、互換性のために残す)
    audit_log: HashMap<dirtydata_core::types::PatchId, TransactionMetadata>,

    /// エンジンへの通信パイプライン
    pipeline: UiToEnginePipeline,

    session_id: String,
}

impl TransactionLayer {
    pub fn new(project: crate::project::ProjectSpec, pipeline: UiToEnginePipeline) -> Self {
        Self {
            project,
            _ongoing_tweaks: HashMap::new(),
            pending_queue: VecDeque::new(),
            history_nodes: Vec::new(),
            current_history_idx: None,
            audit_log: HashMap::new(),
            pipeline,
            session_id: ulid::Ulid::new().to_string(),
        }
    }

    pub fn project(&self) -> &crate::project::ProjectSpec {
        &self.project
    }

    /// UI からの Intent を受け取り、処理を開始する
    pub fn dispatch_intent(&mut self, intent: Intent, author: Author) -> anyhow::Result<()> {
        let trace_id = ulid::Ulid::new();
        info!(session = %self.session_id, %trace_id, %author, "Dispatching intent: {:?}", intent);

        let patch_set = self.compile_intent(&intent, trace_id)?;

        let metadata = TransactionMetadata {
            author,
            timestamp: crate::time::Timestamp::now(),
            intent: intent.clone(),
        };

        if let Some(ps) = patch_set {
            self.pending_queue.push_back(PendingTransaction {
                trace_id,
                metadata,
                _patch_set: Some(ps.clone()),
            });

            // エンジンへ送信
            if let Err(e) = self.pipeline.send(EngineCommand::ApplyPatchSet(ps)) {
                error!("Failed to send patch to engine: {:?}", e);
                return Err(anyhow::anyhow!("Pipeline overflow"));
            }
        }

        Ok(())
    }

    /// Intent を PatchSet に変換する内部ロジック
    fn compile_intent(
        &mut self,
        intent: &Intent,
        trace_id: ulid::Ulid,
    ) -> anyhow::Result<Option<PatchSet>> {
        match intent {
            Intent::AddNode { kind, position } => {
                let _node_id = self.project.ui.add_node(kind, *position);
                let kernel_id = dirtydata_core::StableId::new(); // Bridge to DirtyData

                let node = match kind.as_str() {
                    "Source" => dirtydata_core::Node::new_source("New Source"),
                    "Sink" => dirtydata_core::Node::new_sink("New Sink"),
                    _ => dirtydata_core::Node::new_processor(kind),
                };
                let mut node = node;
                node.id = kernel_id;

                let op = Operation::AddNode(node);
                let mut patch = Patch::from_operations(vec![op]);
                patch.intent_ref = Some(dirtydata_core::IntentId(trace_id));

                Ok(Some(PatchSet::single(patch)))
            }

            Intent::MoveNode { node_id, position } => {
                if let Some(kernel_id) = self.project.view.id_map.resolve(*node_id) {
                    // Perception: Update cached position
                    self.project
                        .view
                        .node_positions
                        .insert(kernel_id, *position);

                    // Reality: Moving nodes usually doesn't affect DSP, but it's part of the topology
                    // In LimeStudio, we record movement as a non-DSP patch if needed.
                    Ok(None)
                } else {
                    Err(anyhow::anyhow!("Invalid UI ID"))
                }
            }

            Intent::TweakParam {
                node_id,
                param,
                value,
            } => {
                if let Some(kernel_id) = self.project.view.id_map.resolve(*node_id) {
                    let mut delta = BTreeMap::new();
                    delta.insert(
                        param.clone(),
                        ConfigChange {
                            old: None,
                            new: Some(ConfigValue::Float(*value as f64)),
                        },
                    );

                    let op = Operation::ModifyConfig {
                        node_id: kernel_id,
                        delta,
                    };
                    let mut patch = Patch::from_operations(vec![op]);
                    patch.intent_ref = Some(dirtydata_core::IntentId(trace_id));

                    Ok(Some(PatchSet::single(patch)))
                } else {
                    Err(anyhow::anyhow!("Invalid UI ID"))
                }
            }

            Intent::EndTweak { .. } => {
                // For now, EndTweak just signifies the end of a continuous interaction.
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    /// エンジンからの応答を処理し、Reconciliation（調停）を行う
    pub fn handle_responses(&mut self, responses: &mut EngineToUiPipeline) {
        while let Some(res) = responses.try_recv() {
            match res {
                EngineResponse::SnapshotUpdated { snapshot, trace_id } => {
                    // Pending Queue から一致するものを探してメタデータを取得
                    if let Some(tid) = trace_id {
                        if let Some(pending) = self.pending_queue.iter().find(|p| p.trace_id == tid)
                        {
                            if let Some(last_patch_id) = snapshot.lineage.applied_patches.last() {
                                let new_node_idx = self.history_nodes.len();

                                let metadata = TransactionMetadata {
                                    author: pending.metadata.author.clone(),
                                    timestamp: pending.metadata.timestamp,
                                    intent: pending.metadata.intent.clone(),
                                };

                                // 監査ログを確定
                                self.audit_log.insert(*last_patch_id, metadata.clone());

                                // 履歴ツリーを成長させる
                                let new_node = HistoryNode {
                                    patch_id: *last_patch_id,
                                    parent: self.current_history_idx,
                                    children: Vec::new(),
                                    project: self.project.clone(),
                                    metadata,
                                };

                                if let Some(parent_idx) = self.current_history_idx {
                                    self.history_nodes[parent_idx].children.push(new_node_idx);
                                }

                                self.history_nodes.push(new_node);
                                self.current_history_idx = Some(new_node_idx);
                            }
                        }
                        self.pending_queue.retain(|p| p.trace_id != tid);
                    }

                    // For now, we don't sync the whole graph back as we want LimeGraph to be SSOT
                    // In a real impl, we'd merge DSP state back into LimeGraph.dsp
                }
                EngineResponse::Error { message, trace_id } => {
                    error!(
                        "Kernel rejected update: {} (TraceID: {:?})",
                        message, trace_id
                    );
                    // ロールバック処理：Pending Queue から削除し、UIを最新の Snapshot で強制同期
                    if let Some(tid) = trace_id {
                        self.pending_queue.retain(|p| p.trace_id != tid);
                    }
                    self.reconcile_ui_state();
                }
                EngineResponse::Telemetry { .. } => {
                    // For now, Telemetry is handled by the UI engine directly or passed through
                    // In a real implementation, we'd update a real-time monitor here.
                }
            }
        }
    }

    /// Undo 操作の実行 (親ノードへ戻る)
    pub fn undo(&mut self) -> anyhow::Result<()> {
        if let Some(idx) = self.current_history_idx {
            if let Some(parent_idx) = self.history_nodes[idx].parent {
                self.jump_to_history(parent_idx)?;
            }
        }
        Ok(())
    }

    /// Redo 操作の実行 (最新の子ノードへ進む)
    pub fn redo(&mut self) -> anyhow::Result<()> {
        if let Some(idx) = self.current_history_idx {
            if let Some(&child_idx) = self.history_nodes[idx].children.last() {
                self.jump_to_history(child_idx)?;
            }
        }
        Ok(())
    }

    /// 履歴ツリーの特定のポイントへジャンプする (タイムトラベル)
    pub fn jump_to_history(&mut self, idx: usize) -> anyhow::Result<()> {
        if let Some(node) = self.history_nodes.get(idx) {
            info!(
                "Time Travel: Jumping to history node {}, patch {:?}",
                idx, node.patch_id
            );
            self.project = node.project.clone();
            self.current_history_idx = Some(idx);
            self.pipeline.send(EngineCommand::Checkout(node.patch_id))?;
        }
        Ok(())
    }

    /// UI状態を Reality (current_graph) に強制同期させる（自己修復）
    fn reconcile_ui_state(&mut self) {
        warn!("Reconciling UI state with Reality...");
        // 1. ViewCache の ID Map を Kernel の現在のグラフに合わせて再構築
        // 2. 存在しないノードの表示位置などを削除
        // 3. 楽観的更新で追加したノードが拒否された場合、ここで消える
    }

    pub fn get_node_lineage(&self, _node_id: StableId) -> Vec<&TransactionMetadata> {
        let mut lineage = Vec::new();
        for patch_id in &self.project.graph.lineage.applied_patches {
            if let Some(meta) = self.audit_log.get(patch_id) {
                // TODO: パッチの内容を精査して、そのノードに関係があるかチェックする
                // 現時点では、全てのメタデータを返すが、将来的にフィルタリングする
                lineage.push(meta);
            }
        }
        lineage
    }

    pub fn view_cache(&self) -> &ViewCache {
        &self.project.view
    }

    pub fn graph(&self) -> &dirtydata_core::ir::Graph {
        &self.project.graph
    }

    pub fn ui_graph(&self) -> &limestudio_graph::LimeGraph {
        &self.project.ui
    }
}
