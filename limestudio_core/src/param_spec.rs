//! Parameter Interaction Spec
//!
//! Widgetは後。ノブの絵は最後。
//! 先に必要なのはParameter Interaction Spec。
//! これが先。

use serde::{Serialize, Deserialize};

// ─── Parameter Definition ──────────────────────────────────

/// パラメータのフルスペック定義
/// UIウィジェットはこのSpecを消費する。Specが先。見た目は後。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParamSpec {
    /// 一意の識別子
    pub id: u32,
    /// 内部名（automation用）
    pub name: String,
    /// 表示名
    pub display_name: String,
    /// 値の範囲と特性
    pub range: ParamRange,
    /// デフォルト値 (normalized 0.0..1.0)
    pub default_normalized: f32,
    /// 表示単位
    pub unit: ParamUnit,
    /// スムージング設定
    pub smoothing: SmoothingSpec,
    /// インタラクション挙動
    pub interaction: InteractionSpec,
    /// オートメーション設定
    pub automation: AutomationSpec,
    /// モジュレーション設定
    pub modulation: ModulationSpec,
}

/// 値の範囲とスケーリング
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParamRange {
    pub min: f32,
    pub max: f32,
    /// スケーリングカーブ
    pub skew: SkewFactor,
    /// ステップ（離散値の場合）
    pub step: Option<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SkewFactor {
    /// 線形
    Linear,
    /// 対数（周波数向け）
    Logarithmic,
    /// 指数（dB向け）
    Exponential(f32),
    /// カスタムカーブ（べき乗）
    Power(f32),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ParamUnit {
    None,
    Decibel,
    Hertz,
    Milliseconds,
    Seconds,
    Percent,
    Semitones,
    Octaves,
    Ratio,
    Degrees,
    Custom(String),
}

// ─── Interaction Spec ──────────────────────────────────────

/// マウス/キーボードのインタラクション挙動定義
/// FabFilterの本質はここ。
/// 情報密度、操作予測性、マウス移動量に対する意味の線形性。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InteractionSpec {
    /// 通常ドラッグ感度 (pixels per full range)
    pub drag_sensitivity: f32,
    /// Shift押下時の細かい調整倍率 (例: 0.1 = 10倍精密)
    pub fine_adjust_factor: f32,
    /// Ctrl/Cmd押下時のスナップ挙動
    pub snap_behavior: SnapBehavior,
    /// ダブルクリックでデフォルト値にリセット
    pub double_click_reset: bool,
    /// 右クリックメニュー項目
    pub context_menu: Vec<ContextMenuItem>,
    /// キーボード入力で数値直接入力を許可
    pub keyboard_input: bool,
    /// マウスホイール感度
    pub scroll_sensitivity: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SnapBehavior {
    /// スナップなし
    None,
    /// 固定グリッドにスナップ
    Grid(f32),
    /// 特定の値リストにスナップ（例: 整数dB, MIDIノート）
    Values(Vec<f32>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ContextMenuItem {
    /// MIDI Learn
    MidiLearn,
    /// パラメータ値のコピー
    CopyValue,
    /// パラメータ値のペースト
    PasteValue,
    /// デフォルトにリセット
    ResetDefault,
    /// モジュレーション追加
    AddModulation,
    /// カスタム項目
    Custom { label: String, action_id: String },
}

// ─── Smoothing ─────────────────────────────────────────────

/// パラメータスムージング設定
/// smoothing missing → zipper noise → 即死
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SmoothingSpec {
    pub style: SmoothingStyle,
    /// スムージング時間 (ms)
    pub time_ms: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SmoothingStyle {
    /// スムージングなし（離散パラメータ向け）
    None,
    /// 線形補間
    Linear,
    /// 指数減衰（最も自然）
    Exponential,
    /// 対数（ゲイン向け）
    Logarithmic,
}

// ─── Automation ────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AutomationSpec {
    /// オートメーション可能か
    pub automatable: bool,
    /// DAWに公開するパラメータ名
    pub daw_name: Option<String>,
    /// パラメータグループ（DAW側での分類）
    pub group: Option<String>,
}

// ─── Modulation ────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModulationSpec {
    /// モジュレーション可能か
    pub modulatable: bool,
    /// バイポーラモジュレーション許可
    pub bipolar: bool,
    /// モジュレーションリング表示（FabFilter的なやつ）
    pub show_ring: bool,
    /// 最大モジュレーションソース数
    pub max_sources: u8,
}

// ─── Defaults ──────────────────────────────────────────────

impl Default for InteractionSpec {
    fn default() -> Self {
        Self {
            drag_sensitivity: 200.0, // 200px for full range
            fine_adjust_factor: 0.1,
            snap_behavior: SnapBehavior::None,
            double_click_reset: true,
            context_menu: vec![
                ContextMenuItem::MidiLearn,
                ContextMenuItem::ResetDefault,
                ContextMenuItem::CopyValue,
                ContextMenuItem::PasteValue,
            ],
            keyboard_input: true,
            scroll_sensitivity: 0.01,
        }
    }
}

impl Default for SmoothingSpec {
    fn default() -> Self {
        Self {
            style: SmoothingStyle::Exponential,
            time_ms: 20.0,
        }
    }
}

impl Default for AutomationSpec {
    fn default() -> Self {
        Self {
            automatable: true,
            daw_name: None,
            group: None,
        }
    }
}

impl Default for ModulationSpec {
    fn default() -> Self {
        Self {
            modulatable: true,
            bipolar: true,
            show_ring: true,
            max_sources: 4,
        }
    }
}

// ─── Builder ───────────────────────────────────────────────

/// よくあるパラメータパターンのファクトリ
impl ParamSpec {
    /// 周波数パラメータ（20Hz - 20kHz, 対数スケール）
    pub fn frequency(id: u32, name: &str, default_hz: f32) -> Self {
        let normalized = ((default_hz / 20.0).ln() / (20000.0 / 20.0_f32).ln()).clamp(0.0, 1.0);
        Self {
            id,
            name: name.to_string(),
            display_name: name.to_string(),
            range: ParamRange {
                min: 20.0,
                max: 20000.0,
                skew: SkewFactor::Logarithmic,
                step: None,
            },
            default_normalized: normalized,
            unit: ParamUnit::Hertz,
            smoothing: SmoothingSpec {
                style: SmoothingStyle::Exponential,
                time_ms: 50.0,
            },
            interaction: InteractionSpec::default(),
            automation: AutomationSpec::default(),
            modulation: ModulationSpec::default(),
        }
    }

    /// ゲインパラメータ (-inf to +12dB)
    pub fn gain_db(id: u32, name: &str, default_db: f32) -> Self {
        let normalized = ((default_db + 96.0) / 108.0).clamp(0.0, 1.0);
        Self {
            id,
            name: name.to_string(),
            display_name: name.to_string(),
            range: ParamRange {
                min: -96.0,
                max: 12.0,
                skew: SkewFactor::Exponential(3.0),
                step: None,
            },
            default_normalized: normalized,
            unit: ParamUnit::Decibel,
            smoothing: SmoothingSpec::default(),
            interaction: InteractionSpec {
                snap_behavior: SnapBehavior::Grid(1.0), // 1dBスナップ
                ..InteractionSpec::default()
            },
            automation: AutomationSpec::default(),
            modulation: ModulationSpec::default(),
        }
    }

    /// パーセントパラメータ (0% - 100%)
    pub fn percent(id: u32, name: &str, default_pct: f32) -> Self {
        Self {
            id,
            name: name.to_string(),
            display_name: name.to_string(),
            range: ParamRange {
                min: 0.0,
                max: 100.0,
                skew: SkewFactor::Linear,
                step: None,
            },
            default_normalized: default_pct / 100.0,
            unit: ParamUnit::Percent,
            smoothing: SmoothingSpec::default(),
            interaction: InteractionSpec::default(),
            automation: AutomationSpec::default(),
            modulation: ModulationSpec::default(),
        }
    }

    /// 時間パラメータ (ms)
    pub fn time_ms(id: u32, name: &str, min_ms: f32, max_ms: f32, default_ms: f32) -> Self {
        let normalized = ((default_ms - min_ms) / (max_ms - min_ms)).clamp(0.0, 1.0);
        Self {
            id,
            name: name.to_string(),
            display_name: name.to_string(),
            range: ParamRange {
                min: min_ms,
                max: max_ms,
                skew: SkewFactor::Logarithmic,
                step: None,
            },
            default_normalized: normalized,
            unit: ParamUnit::Milliseconds,
            smoothing: SmoothingSpec {
                style: SmoothingStyle::Linear,
                time_ms: 5.0,
            },
            interaction: InteractionSpec::default(),
            automation: AutomationSpec::default(),
            modulation: ModulationSpec::default(),
        }
    }

    /// Normalized (0.0..1.0) → 実値に変換
    pub fn denormalize(&self, normalized: f32) -> f32 {
        let n = normalized.clamp(0.0, 1.0);
        match &self.range.skew {
            SkewFactor::Linear => {
                self.range.min + n * (self.range.max - self.range.min)
            }
            SkewFactor::Logarithmic => {
                self.range.min * (self.range.max / self.range.min).powf(n)
            }
            SkewFactor::Exponential(exp) => {
                self.range.min + n.powf(*exp) * (self.range.max - self.range.min)
            }
            SkewFactor::Power(p) => {
                self.range.min + n.powf(*p) * (self.range.max - self.range.min)
            }
        }
    }

    /// 実値 → Normalized (0.0..1.0) に変換
    pub fn normalize(&self, value: f32) -> f32 {
        let v = value.clamp(self.range.min, self.range.max);
        match &self.range.skew {
            SkewFactor::Linear => {
                (v - self.range.min) / (self.range.max - self.range.min)
            }
            SkewFactor::Logarithmic => {
                (v / self.range.min).ln() / (self.range.max / self.range.min).ln()
            }
            SkewFactor::Exponential(exp) => {
                ((v - self.range.min) / (self.range.max - self.range.min)).powf(1.0 / exp)
            }
            SkewFactor::Power(p) => {
                ((v - self.range.min) / (self.range.max - self.range.min)).powf(1.0 / p)
            }
        }
    }

    /// 表示用文字列を生成
    pub fn format_value(&self, normalized: f32) -> String {
        let value = self.denormalize(normalized);
        match &self.unit {
            ParamUnit::Decibel => {
                if value <= -95.0 { "-∞ dB".to_string() }
                else { format!("{:.1} dB", value) }
            }
            ParamUnit::Hertz => {
                if value >= 1000.0 { format!("{:.2} kHz", value / 1000.0) }
                else { format!("{:.1} Hz", value) }
            }
            ParamUnit::Milliseconds => format!("{:.1} ms", value),
            ParamUnit::Seconds => format!("{:.2} s", value),
            ParamUnit::Percent => format!("{:.1}%", value),
            ParamUnit::Semitones => format!("{:.1} st", value),
            ParamUnit::Octaves => format!("{:.2} oct", value),
            ParamUnit::Ratio => format!("{:.1}:1", value),
            ParamUnit::Degrees => format!("{:.0}°", value),
            ParamUnit::None => format!("{:.2}", value),
            ParamUnit::Custom(u) => format!("{:.2} {}", value, u),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frequency_roundtrip() {
        let spec = ParamSpec::frequency(0, "Freq", 1000.0);
        let norm = spec.normalize(1000.0);
        let denorm = spec.denormalize(norm);
        assert!((denorm - 1000.0).abs() < 1.0);
    }

    #[test]
    fn test_gain_format() {
        let spec = ParamSpec::gain_db(0, "Gain", 0.0);
        assert_eq!(spec.format_value(spec.normalize(-96.0)), "-∞ dB");
        let s = spec.format_value(spec.normalize(0.0));
        assert!(s.contains("0.0 dB"));
    }

    #[test]
    fn test_linear_normalize() {
        let spec = ParamSpec::percent(0, "Mix", 50.0);
        assert!((spec.denormalize(0.0) - 0.0).abs() < 0.01);
        assert!((spec.denormalize(1.0) - 100.0).abs() < 0.01);
        assert!((spec.denormalize(0.5) - 50.0).abs() < 0.01);
    }
}
