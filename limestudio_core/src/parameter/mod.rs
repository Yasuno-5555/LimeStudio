use serde::{Serialize, Deserialize};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use crossbeam_channel::{Sender, Receiver}; // 需要追加 dependency

/// パラメータ変更イベント
#[derive(Clone, Debug)]
pub struct ParamChangeEvent {
    pub id: StableParamId,
    pub normalized_value: f32,
    pub source: ChangeSource,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChangeSource {
    Host,
    UI,
    Internal, // Modulation etc.
}

/// 永続的なパラメータ識別子
/// JUCEの「ParamID」に相当。名前が変わってもマイグレーションマップで追跡可能。
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StableParamId(pub String);

/// パラメータの表示単位
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ParamUnit {
    Generic,
    Hertz,
    Decibels,
    Percent,
    Milliseconds,
    Semitones,
}

/// パラメータのカーブ（正規化された 0.0..1.0 から実値への変換）
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ParamSkew {
    Linear,
    Logarithmic,
    Exponential,
}

/// パラメータの静的定義 (S Tier: S1 & S2)
/// プロ品質のプラグインに必須のメタデータ
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParameterDefinition {
    pub id: StableParamId,
    pub name: String,
    pub default_value: f32, // 0.0 to 1.0
    pub unit: ParamUnit,
    pub skew: ParamSkew,
    pub range: (f32, f32),
    
    /// マイグレーション用：このパラメータの古い名前
    pub legacy_ids: Vec<String>,
}

/// リアルタイム実行用のパラメータ状態 (S1: Automation & Thread Safety)
pub struct ManagedParameter {
    pub definition: Arc<ParameterDefinition>,
    /// 正規化された現在の値 (0.0 .. 1.0)
    /// UI/Hostスレッドから書き込み、Audioスレッドから読み込む
    value: AtomicU32,
    /// 前回の値（平滑化や差分検知用）
    last_value: AtomicU32,
}

impl ManagedParameter {
    pub fn new(def: ParameterDefinition) -> Self {
        let val = def.default_value;
        Self {
            definition: Arc::new(def),
            value: AtomicU32::new(val.to_bits()),
            last_value: AtomicU32::new(val.to_bits()),
        }
    }

    /// ホストまたはUIからの値をセット (Normalized 0.0..1.0)
    pub fn set_normalized(&self, val: f32) {
        let clamped = val.clamp(0.0, 1.0);
        self.value.store(clamped.to_bits(), Ordering::Relaxed);
    }

    /// 正規化された現在の値を取得 (Real-time safe)
    pub fn get_normalized(&self) -> f32 {
        f32::from_bits(self.value.load(Ordering::Relaxed))
    }

    /// スキュー（カーブ）を適用した実値を取得
    pub fn get_plain_value(&self) -> f32 {
        let norm = self.get_normalized();
        let (min, max) = self.definition.range;
        
        match self.definition.skew {
            ParamSkew::Linear => min + (max - min) * norm,
            ParamSkew::Logarithmic => {
                // Simplified log mapping
                let min_log = (min.max(1e-6)).ln();
                let max_log = (max.max(1e-6)).ln();
                (min_log + (max_log - min_log) * norm).exp()
            },
            ParamSkew::Exponential => {
                min + (max - min) * norm.powf(2.0)
            }
        }
    }
}

/// パラメータ・レジストリ (S Tier: S1 & S2)
/// 全てのパラメータの定義とマイグレーションを一括管理
pub struct ParameterRegistry {
    pub parameters: Vec<Arc<ManagedParameter>>,
    /// UIやホストへの変更通知用キュー
    change_sender: Sender<ParamChangeEvent>,
    change_receiver: Receiver<ParamChangeEvent>,
}

impl Default for ParameterRegistry {
    fn default() -> Self {
        let (s, r) = crossbeam_channel::unbounded();
        Self {
            parameters: Vec::new(),
            change_sender: s,
            change_receiver: r,
        }
    }
}

impl ParameterRegistry {
    pub fn add(&mut self, def: ParameterDefinition) {
        self.parameters.push(Arc::new(ManagedParameter::new(def)));
    }

    pub fn find_by_id(&self, id: &str) -> Option<Arc<ManagedParameter>> {
        self.parameters.iter().find(|p| p.definition.id.0 == id).cloned()
    }

    /// 値を更新し、通知を送る
    pub fn update_parameter(&self, id: &str, val: f32, source: ChangeSource) {
        if let Some(param) = self.find_by_id(id) {
            param.set_normalized(val);
            let _ = self.change_sender.try_send(ParamChangeEvent {
                id: StableParamId(id.to_string()),
                normalized_value: val,
                source,
            });
        }
    }

    /// 変更通知を受信 (UIスレッド等で呼ぶ)
    pub fn poll_changes(&self) -> Vec<ParamChangeEvent> {
        self.change_receiver.try_iter().collect()
    }

    /// シリアライズ用 (S Tier: S1)
    pub fn serialize_values(&self) -> Vec<(String, f32)> {
        self.parameters.iter()
            .map(|p| (p.definition.id.0.clone(), p.get_normalized()))
            .collect()
    }

    /// デシリアライズ & マイグレーション (S Tier: S2)
    /// 古いIDを現在のパラメータにマッピングして値を復元
    pub fn load_values(&self, data: Vec<(String, f32)>) {
        for (id, val) in data {
            if let Some(param) = self.find_by_id(&id) {
                param.set_normalized(val);
            } else {
                // Legacy ID search
                if let Some(param) = self.parameters.iter().find(|p| p.definition.legacy_ids.contains(&id)) {
                    param.set_normalized(val);
                }
            }
        }
    }
}
