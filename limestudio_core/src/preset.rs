//! Preset / State / Migration System
//!
//! schema_version: u32 — プリセットフォーマットの互換性
//! plugin_version: semver — バイナリのリリースバージョン
//! これらは別物。混ぜると半年後の自分が刺しに来る。

use serde::{Serialize, Deserialize};
use crate::graph::AudioGraph;
use std::collections::HashMap;

/// 現在のスキーマバージョン
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

// ─── Preset ────────────────────────────────────────────────

/// プリセットの完全な状態
/// graph丸ごと保存だけでは足りない。
/// A/B compare, undo/redo, preset morph — 全部ここから始まる。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Preset {
    pub metadata: PresetMetadata,
    pub parameters: ParameterState,
    pub modulation_state: ModulationState,
    pub graph_snapshot: AudioGraph,
    pub ui_state: UiState,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PresetMetadata {
    /// プリセットフォーマットの互換性バージョン (u32)
    pub schema_version: u32,
    /// プラグインのリリースバージョン (semver string)
    pub plugin_version: String,
    /// プリセット名
    pub name: String,
    /// 作者
    pub author: String,
    /// カテゴリタグ
    pub tags: Vec<String>,
    /// 説明
    pub description: String,
    /// 作成日時 (ISO 8601)
    pub created_at: String,
    /// 最終更新日時
    pub modified_at: String,
}

/// 全パラメータの値スナップショット
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParameterState {
    /// param_id -> normalized value (0.0..1.0)
    pub values: HashMap<u32, f32>,
}

/// モジュレーションマトリクスの状態
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModulationState {
    pub routings: Vec<ModulationRouting>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModulationRouting {
    pub source_id: u32,
    pub target_param_id: u32,
    pub amount: f32,
    pub bipolar: bool,
}

/// UIレイアウト状態（ウィンドウサイズ、スクロール位置など）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UiState {
    pub window_width: u32,
    pub window_height: u32,
    pub scroll_x: f32,
    pub scroll_y: f32,
    pub zoom: f32,
    /// 任意のUI状態を保存
    pub extra: HashMap<String, String>,
}

// ─── Defaults ──────────────────────────────────────────────

impl Default for PresetMetadata {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            plugin_version: String::from("0.1.0"),
            name: String::from("Init"),
            author: String::new(),
            tags: Vec::new(),
            description: String::new(),
            created_at: String::new(),
            modified_at: String::new(),
        }
    }
}

impl Default for ParameterState {
    fn default() -> Self {
        Self { values: HashMap::new() }
    }
}

impl Default for ModulationState {
    fn default() -> Self {
        Self { routings: Vec::new() }
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            window_width: 1200,
            window_height: 800,
            scroll_x: 0.0,
            scroll_y: 0.0,
            zoom: 1.0,
            extra: HashMap::new(),
        }
    }
}

impl Default for Preset {
    fn default() -> Self {
        Self {
            metadata: PresetMetadata::default(),
            parameters: ParameterState::default(),
            modulation_state: ModulationState::default(),
            graph_snapshot: AudioGraph::new(),
            ui_state: UiState::default(),
        }
    }
}

// ─── Serialization ─────────────────────────────────────────

impl Preset {
    /// JSON にシリアライズ
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// JSON からデシリアライズ（マイグレーション付き）
    pub fn from_json(json: &str) -> Result<Self, PresetError> {
        // まずスキーマバージョンだけ読む
        let raw: serde_json::Value = serde_json::from_str(json)
            .map_err(PresetError::JsonError)?;

        let schema_version = raw
            .get("metadata")
            .and_then(|m| m.get("schema_version"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        if schema_version > CURRENT_SCHEMA_VERSION {
            return Err(PresetError::FutureSchema {
                found: schema_version,
                current: CURRENT_SCHEMA_VERSION,
            });
        }

        // マイグレーション適用
        let migrated = migrate(raw, schema_version)?;

        serde_json::from_value(migrated).map_err(PresetError::JsonError)
    }

    /// .lime ファイルに保存
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<(), PresetError> {
        let json = self.to_json().map_err(PresetError::JsonError)?;
        std::fs::write(path, json).map_err(PresetError::IoError)
    }

    /// .lime ファイルから読み込み
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, PresetError> {
        let json = std::fs::read_to_string(path).map_err(PresetError::IoError)?;
        Self::from_json(&json)
    }
}

// ─── Error ─────────────────────────────────────────────────

#[derive(Debug)]
pub enum PresetError {
    JsonError(serde_json::Error),
    IoError(std::io::Error),
    FutureSchema { found: u32, current: u32 },
    MigrationFailed { from: u32, to: u32, reason: String },
}

impl std::fmt::Display for PresetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PresetError::JsonError(e) => write!(f, "JSON error: {}", e),
            PresetError::IoError(e) => write!(f, "IO error: {}", e),
            PresetError::FutureSchema { found, current } =>
                write!(f, "Preset schema v{} is newer than supported v{}", found, current),
            PresetError::MigrationFailed { from, to, reason } =>
                write!(f, "Migration v{} -> v{} failed: {}", from, to, reason),
        }
    }
}

impl std::error::Error for PresetError {}

// ─── Migration ─────────────────────────────────────────────

/// スキーマバージョンを段階的にマイグレーションする
fn migrate(mut value: serde_json::Value, from_version: u32) -> Result<serde_json::Value, PresetError> {
    let mut current = from_version;

    while current < CURRENT_SCHEMA_VERSION {
        value = match current {
            0 => migrate_0_to_1(value)?,
            _ => return Err(PresetError::MigrationFailed {
                from: current,
                to: current + 1,
                reason: "No migration path defined".into(),
            }),
        };
        current += 1;
    }

    Ok(value)
}

/// v0 (schema_version未定義の古いフォーマット) → v1
fn migrate_0_to_1(mut value: serde_json::Value) -> Result<serde_json::Value, PresetError> {
    // 古いフォーマット: graph だけが保存されていた場合の救済
    if let Some(metadata) = value.get_mut("metadata") {
        if let Some(obj) = metadata.as_object_mut() {
            obj.insert("schema_version".into(), serde_json::json!(1));
            if !obj.contains_key("tags") {
                obj.insert("tags".into(), serde_json::json!([]));
            }
            if !obj.contains_key("description") {
                obj.insert("description".into(), serde_json::json!(""));
            }
        }
    }
    // modulation_state が無い場合のデフォルト
    if value.get("modulation_state").is_none() {
        value.as_object_mut().unwrap().insert(
            "modulation_state".into(),
            serde_json::json!({"routings": []}),
        );
    }
    // ui_state が無い場合のデフォルト
    if value.get("ui_state").is_none() {
        value.as_object_mut().unwrap().insert(
            "ui_state".into(),
            serde_json::json!({
                "window_width": 1200, "window_height": 800,
                "scroll_x": 0.0, "scroll_y": 0.0, "zoom": 1.0,
                "extra": {}
            }),
        );
    }
    Ok(value)
}

// ─── A/B Compare ───────────────────────────────────────────

/// A/B比較用のプリセットスロット
pub struct ABSlot {
    pub a: Preset,
    pub b: Preset,
    pub active: ABState,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ABState {
    A,
    B,
}

impl ABSlot {
    pub fn new(initial: Preset) -> Self {
        Self {
            a: initial.clone(),
            b: initial,
            active: ABState::A,
        }
    }

    /// 現在アクティブなプリセットを返す
    pub fn current(&self) -> &Preset {
        match self.active {
            ABState::A => &self.a,
            ABState::B => &self.b,
        }
    }

    /// 現在アクティブなプリセットを可変で返す
    pub fn current_mut(&mut self) -> &mut Preset {
        match self.active {
            ABState::A => &mut self.a,
            ABState::B => &mut self.b,
        }
    }

    /// A/B切り替え
    pub fn toggle(&mut self) {
        self.active = match self.active {
            ABState::A => ABState::B,
            ABState::B => ABState::A,
        };
    }

    /// 現在の状態を非アクティブ側にコピー
    pub fn copy_to_inactive(&mut self) {
        match self.active {
            ABState::A => self.b = self.a.clone(),
            ABState::B => self.a = self.b.clone(),
        }
    }
}

// ─── Undo/Redo ─────────────────────────────────────────────

/// リングバッファ方式のUndo/Redo
pub struct UndoHistory {
    states: Vec<Preset>,
    cursor: usize,
    max_depth: usize,
}

impl UndoHistory {
    pub fn new(initial: Preset, max_depth: usize) -> Self {
        Self {
            states: vec![initial],
            cursor: 0,
            max_depth,
        }
    }

    /// 新しい状態をプッシュ（cursor以降は破棄）
    pub fn push(&mut self, state: Preset) {
        self.states.truncate(self.cursor + 1);
        self.states.push(state);
        if self.states.len() > self.max_depth {
            self.states.remove(0);
        } else {
            self.cursor += 1;
        }
    }

    pub fn undo(&mut self) -> Option<&Preset> {
        if self.cursor > 0 {
            self.cursor -= 1;
            Some(&self.states[self.cursor])
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<&Preset> {
        if self.cursor + 1 < self.states.len() {
            self.cursor += 1;
            Some(&self.states[self.cursor])
        } else {
            None
        }
    }

    pub fn current(&self) -> &Preset {
        &self.states[self.cursor]
    }

    pub fn can_undo(&self) -> bool {
        self.cursor > 0
    }

    pub fn can_redo(&self) -> bool {
        self.cursor + 1 < self.states.len()
    }
}

// ─── Preset Doctor ─────────────────────────────────────────

/// preset-doctor: プリセットの健全性チェック
#[derive(Debug)]
pub struct DiagnosticReport {
    pub issues: Vec<DiagnosticIssue>,
}

#[derive(Debug)]
pub struct DiagnosticIssue {
    pub severity: Severity,
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl DiagnosticReport {
    pub fn has_errors(&self) -> bool {
        self.issues.iter().any(|i| i.severity == Severity::Error)
    }
}

/// プリセットを診断する
pub fn diagnose_preset(preset: &Preset) -> DiagnosticReport {
    let mut issues = Vec::new();

    // 1. Schema version check
    if preset.metadata.schema_version > CURRENT_SCHEMA_VERSION {
        issues.push(DiagnosticIssue {
            severity: Severity::Error,
            code: "E001",
            message: format!(
                "Schema version {} is newer than supported {}",
                preset.metadata.schema_version, CURRENT_SCHEMA_VERSION
            ),
        });
    }

    // 2. Empty name
    if preset.metadata.name.trim().is_empty() {
        issues.push(DiagnosticIssue {
            severity: Severity::Warning,
            code: "W001",
            message: "Preset name is empty".into(),
        });
    }

    // 3. Graph has no nodes
    if preset.graph_snapshot.nodes.is_empty() {
        issues.push(DiagnosticIssue {
            severity: Severity::Warning,
            code: "W002",
            message: "Graph snapshot is empty (no nodes)".into(),
        });
    }

    // 4. Graph validation
    match crate::validate::validate_graph(&preset.graph_snapshot) {
        Ok(_) => {}
        Err(e) => {
            issues.push(DiagnosticIssue {
                severity: Severity::Error,
                code: "E002",
                message: format!("Graph validation failed: {:?}", e),
            });
        }
    }

    // 5. Modulation targets reference valid params
    for routing in &preset.modulation_state.routings {
        if !preset.parameters.values.contains_key(&routing.target_param_id) {
            issues.push(DiagnosticIssue {
                severity: Severity::Warning,
                code: "W003",
                message: format!(
                    "Modulation target param {} not found in parameter state",
                    routing.target_param_id
                ),
            });
        }
    }

    // 6. Deprecated node check (placeholder for future StdlibNode deprecation)
    for node in &preset.graph_snapshot.nodes {
        if let crate::graph::GraphNode::Script { source, .. } = node {
            if source.contains("deprecated") {
                issues.push(DiagnosticIssue {
                    severity: Severity::Warning,
                    code: "W004",
                    message: "Script contains reference to deprecated API".into(),
                });
            }
        }
    }

    DiagnosticReport { issues }
}

// ─── Tests ─────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{AudioGraph, GraphNode};

    fn make_test_preset() -> Preset {
        let mut graph = AudioGraph::new();
        let in_n = graph.add_node(GraphNode::Input { channel: 0 });
        let out_n = graph.add_node(GraphNode::Output { channel: 0 });
        graph.add_edge(in_n, 0, out_n, 0);

        let mut params = ParameterState::default();
        params.values.insert(0, 0.5);
        params.values.insert(1, 1.0);

        Preset {
            metadata: PresetMetadata {
                name: "Test Preset".into(),
                author: "Unit Test".into(),
                ..PresetMetadata::default()
            },
            parameters: params,
            modulation_state: ModulationState::default(),
            graph_snapshot: graph,
            ui_state: UiState::default(),
        }
    }

    #[test]
    fn test_roundtrip() {
        let preset = make_test_preset();
        let json = preset.to_json().unwrap();
        let restored = Preset::from_json(&json).unwrap();
        assert_eq!(restored.metadata.name, "Test Preset");
        assert_eq!(restored.parameters.values.get(&0), Some(&0.5));
        assert_eq!(restored.graph_snapshot.nodes.len(), 2);
    }

    #[test]
    fn test_future_schema_rejected() {
        let mut preset = make_test_preset();
        preset.metadata.schema_version = 9999;
        let json = preset.to_json().unwrap();
        let result = Preset::from_json(&json);
        assert!(matches!(result, Err(PresetError::FutureSchema { .. })));
    }

    #[test]
    fn test_ab_compare() {
        let mut ab = ABSlot::new(make_test_preset());
        assert_eq!(ab.active, ABState::A);

        ab.current_mut().metadata.name = "Modified A".into();
        ab.toggle();
        assert_eq!(ab.current().metadata.name, "Test Preset");
        ab.toggle();
        assert_eq!(ab.current().metadata.name, "Modified A");
    }

    #[test]
    fn test_undo_redo() {
        let mut history = UndoHistory::new(make_test_preset(), 50);
        assert!(!history.can_undo());
        assert!(!history.can_redo());

        let mut p2 = make_test_preset();
        p2.metadata.name = "State 2".into();
        history.push(p2);
        assert!(history.can_undo());

        let undone = history.undo().unwrap();
        assert_eq!(undone.metadata.name, "Test Preset");
        assert!(history.can_redo());

        let redone = history.redo().unwrap();
        assert_eq!(redone.metadata.name, "State 2");
    }

    #[test]
    fn test_doctor_clean() {
        let preset = make_test_preset();
        let report = diagnose_preset(&preset);
        assert!(!report.has_errors());
    }

    #[test]
    fn test_doctor_catches_empty_graph() {
        let preset = Preset::default();
        let report = diagnose_preset(&preset);
        assert!(report.issues.iter().any(|i| i.code == "W002"));
    }

    #[test]
    fn test_doctor_catches_broken_modulation() {
        let mut preset = make_test_preset();
        preset.modulation_state.routings.push(ModulationRouting {
            source_id: 0,
            target_param_id: 999, // doesn't exist in parameters
            amount: 0.5,
            bipolar: false,
        });
        let report = diagnose_preset(&preset);
        assert!(report.issues.iter().any(|i| i.code == "W003"));
    }
}
