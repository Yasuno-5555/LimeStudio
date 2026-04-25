//! ParamGraph — k-rate / a-rate / event-rate の3層分離
//!
//! Audio Graph — サンプル単位の信号処理
//! Control Graph — ブロック単位のモジュレーション (k-rate)
//! Parameter Graph — イベント単位のパラメータ変更 (event-rate)
//!
//! これを分離しないと MassiveにもSerumにもなれない。

use serde::{Serialize, Deserialize};
use crate::ir::ParamId;

// ─── Rate Classification ───────────────────────────────────

/// 処理レート分類
/// これが全てのパラメータ・モジュレーション設計の基盤
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProcessRate {
    /// a-rate: サンプル単位 (44100Hz / 48000Hz)
    /// 用途: オシレータ周波数, フィルタカットオフ (FM/AM)
    Audio,
    /// k-rate: ブロック単位 (一般的に64-512サンプルごと)
    /// 用途: LFO, エンベロープ, ほとんどのモジュレーション
    Control,
    /// event-rate: 不定期 (MIDI, オートメーション, UI操作)
    /// 用途: ノートオン/オフ, プリセット変更, UIノブ操作
    Event,
}

// ─── Modulation Source ─────────────────────────────────────

/// モジュレーションソースの定義
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModSource {
    pub id: u32,
    pub name: String,
    pub rate: ProcessRate,
    pub kind: ModSourceKind,
    /// 出力範囲 (通常 0..1 or -1..1)
    pub bipolar: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ModSourceKind {
    /// LFO (k-rate)
    Lfo { shape: LfoShape, sync: TempoSync },
    /// エンベロープ (k-rate)
    Envelope { stages: u8 },
    /// ベロシティ (event-rate)
    Velocity,
    /// キートラッキング (event-rate)
    KeyTrack,
    /// アフタータッチ (k-rate)
    Aftertouch,
    /// モッドホイール (k-rate)
    ModWheel,
    /// マクロ (event-rate, UIから)
    Macro { index: u8 },
    /// 外部サイドチェイン (a-rate)
    Sidechain { channel: u8 },
    /// カスタム (グラフからの信号)
    Custom,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum LfoShape {
    Sine,
    Triangle,
    Saw,
    Square,
    SampleAndHold,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TempoSync {
    Free { hz: f32 },
    Synced { division: TempoDivision },
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum TempoDivision {
    Whole,
    Half,
    Quarter,
    Eighth,
    Sixteenth,
    DottedQuarter,
    DottedEighth,
    TripletQuarter,
    TripletEighth,
}

// ─── Modulation Routing ────────────────────────────────────

/// モジュレーションルーティング（ソース→ターゲット）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModRouting {
    pub source_id: u32,
    pub target_param: ParamId,
    /// モジュレーション量 (-1.0 to 1.0)
    pub amount: f32,
    /// ルーティングの有効レート（ソースとターゲットの低い方に合わせる）
    pub effective_rate: ProcessRate,
}

// ─── ParamGraph ────────────────────────────────────────────

/// パラメータグラフ: 全パラメータとモジュレーションの管理
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParamGraph {
    /// モジュレーションソース一覧
    pub sources: Vec<ModSource>,
    /// ルーティング一覧
    pub routings: Vec<ModRouting>,
    /// パラメータごとのレート分類
    pub param_rates: Vec<(ParamId, ProcessRate)>,
}

impl ParamGraph {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
            routings: Vec::new(),
            param_rates: Vec::new(),
        }
    }

    /// モジュレーションソースを追加
    pub fn add_source(&mut self, source: ModSource) -> u32 {
        let id = source.id;
        self.sources.push(source);
        id
    }

    /// ルーティングを追加
    pub fn add_routing(&mut self, routing: ModRouting) {
        self.routings.push(routing);
    }

    /// パラメータのレートを設定
    pub fn set_param_rate(&mut self, param: ParamId, rate: ProcessRate) {
        if let Some(entry) = self.param_rates.iter_mut().find(|(p, _)| *p == param) {
            entry.1 = rate;
        } else {
            self.param_rates.push((param, rate));
        }
    }

    /// 指定パラメータへのモジュレーションソース一覧を取得
    pub fn get_mod_sources_for(&self, param: ParamId) -> Vec<(&ModSource, &ModRouting)> {
        self.routings.iter()
            .filter(|r| r.target_param == param)
            .filter_map(|r| {
                self.sources.iter()
                    .find(|s| s.id == r.source_id)
                    .map(|s| (s, r))
            })
            .collect()
    }

    /// 指定パラメータの実効レートを計算
    /// (モジュレーションされている場合、ソースのレートで決まる)
    pub fn effective_rate(&self, param: ParamId) -> ProcessRate {
        let base = self.param_rates.iter()
            .find(|(p, _)| *p == param)
            .map(|(_, r)| *r)
            .unwrap_or(ProcessRate::Event);

        // モジュレーションされている場合、最も高いレートに引き上げ
        let mod_rate = self.routings.iter()
            .filter(|r| r.target_param == param)
            .filter_map(|r| self.sources.iter().find(|s| s.id == r.source_id))
            .map(|s| s.rate)
            .fold(base, |acc, r| rate_max(acc, r));

        mod_rate
    }

    /// バリデーション: a-rateパラメータにevent-rateソースが繋がっていないかなど
    pub fn validate(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        for routing in &self.routings {
            let source = self.sources.iter().find(|s| s.id == routing.source_id);
            if source.is_none() {
                warnings.push(format!(
                    "Routing references non-existent source {}",
                    routing.source_id
                ));
                continue;
            }

            // amount が 0 のルーティングは無駄
            if routing.amount.abs() < 1e-6 {
                warnings.push(format!(
                    "Routing source {} → param {} has near-zero amount",
                    routing.source_id, routing.target_param
                ));
            }
        }

        warnings
    }
}

/// 2つのレートのうち高い方を返す (Audio > Control > Event)
fn rate_max(a: ProcessRate, b: ProcessRate) -> ProcessRate {
    match (a, b) {
        (ProcessRate::Audio, _) | (_, ProcessRate::Audio) => ProcessRate::Audio,
        (ProcessRate::Control, _) | (_, ProcessRate::Control) => ProcessRate::Control,
        _ => ProcessRate::Event,
    }
}

// ─── Tests ─────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::ParamId;

    #[test]
    fn test_rate_escalation() {
        let mut pg = ParamGraph::new();

        let cutoff = ParamId(0);
        pg.set_param_rate(cutoff, ProcessRate::Event); // UIノブ = event-rate

        // LFOを追加 (k-rate)
        pg.add_source(ModSource {
            id: 0,
            name: "LFO 1".into(),
            rate: ProcessRate::Control,
            kind: ModSourceKind::Lfo {
                shape: LfoShape::Sine,
                sync: TempoSync::Free { hz: 2.0 },
            },
            bipolar: true,
        });

        // LFO → Cutoff
        pg.add_routing(ModRouting {
            source_id: 0,
            target_param: cutoff,
            amount: 0.5,
            effective_rate: ProcessRate::Control,
        });

        // Event-rateパラメータがControl-rateに昇格するはず
        assert_eq!(pg.effective_rate(cutoff), ProcessRate::Control);
    }

    #[test]
    fn test_audio_rate_sidechain() {
        let mut pg = ParamGraph::new();
        let gain = ParamId(1);
        pg.set_param_rate(gain, ProcessRate::Event);

        pg.add_source(ModSource {
            id: 1,
            name: "Sidechain".into(),
            rate: ProcessRate::Audio,
            kind: ModSourceKind::Sidechain { channel: 2 },
            bipolar: false,
        });

        pg.add_routing(ModRouting {
            source_id: 1,
            target_param: gain,
            amount: 1.0,
            effective_rate: ProcessRate::Audio,
        });

        // Audio-rateソースで最高レートに
        assert_eq!(pg.effective_rate(gain), ProcessRate::Audio);
    }

    #[test]
    fn test_validation_catches_zero_amount() {
        let mut pg = ParamGraph::new();
        pg.add_source(ModSource {
            id: 0,
            name: "LFO".into(),
            rate: ProcessRate::Control,
            kind: ModSourceKind::Lfo {
                shape: LfoShape::Sine,
                sync: TempoSync::Free { hz: 1.0 },
            },
            bipolar: true,
        });
        pg.add_routing(ModRouting {
            source_id: 0,
            target_param: ParamId(0),
            amount: 0.0, // useless
            effective_rate: ProcessRate::Control,
        });

        let warnings = pg.validate();
        assert!(!warnings.is_empty());
    }
}
