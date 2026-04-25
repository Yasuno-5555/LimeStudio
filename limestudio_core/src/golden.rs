//! Golden Audio Tests
//!
//! 同じpresetから同じwavが出ること。
//! 製品はここ。思想ではない。波形。無慈悲。

use crate::engine::DspEngine;
use crate::compile::CompiledGraph;
use crate::preset::Preset;

/// Golden Test の結果
#[derive(Debug)]
pub struct GoldenTestResult {
    pub passed: bool,
    pub max_deviation: f32,
    pub rms_deviation: f32,
    pub sample_count: usize,
    /// 最初に不一致が検出されたサンプル位置
    pub first_mismatch_at: Option<usize>,
}

/// プリセットからオーディオをレンダリング（オフライン）
pub fn render_preset(
    preset: &Preset,
    sample_rate: u32,
    num_samples: usize,
    input: &[f32],
) -> Result<Vec<f32>, String> {
    let engine = DspEngine::new(&preset.graph_snapshot)?;
    render_with_engine(engine, sample_rate, num_samples, input)
}

/// エンジンからオーディオをレンダリング
fn render_with_engine(
    mut engine: DspEngine,
    sample_rate: u32,
    num_samples: usize,
    input: &[f32],
) -> Result<Vec<f32>, String> {
    engine.set_sample_rate(sample_rate);

    let mut output = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let in_sample = if i < input.len() { input[i] } else { 0.0 };
        let (out_l, _out_r) = engine.process_sample(in_sample, in_sample);
        output.push(out_l);
    }

    Ok(output)
}

/// CompiledGraphからオーディオをレンダリング
pub fn render_program(
    program: CompiledGraph,
    sample_rate: u32,
    num_samples: usize,
    input: &[f32],
) -> Vec<f32> {
    let mut engine = DspEngine::new_from_program(program);
    engine.set_sample_rate(sample_rate);

    let mut output = Vec::with_capacity(num_samples);
    for i in 0..num_samples {
        let in_sample = if i < input.len() { input[i] } else { 0.0 };
        let (out_l, _) = engine.process_sample(in_sample, in_sample);
        output.push(out_l);
    }
    output
}

/// 2つの波形を比較
pub fn compare_audio(
    expected: &[f32],
    actual: &[f32],
    tolerance: f32,
) -> GoldenTestResult {
    let len = expected.len().min(actual.len());
    let mut max_dev: f32 = 0.0;
    let mut sum_sq: f64 = 0.0;
    let mut first_mismatch: Option<usize> = None;

    for i in 0..len {
        let dev = (expected[i] - actual[i]).abs();
        if dev > max_dev { max_dev = dev; }
        sum_sq += (dev as f64) * (dev as f64);

        if dev > tolerance && first_mismatch.is_none() {
            first_mismatch = Some(i);
        }
    }

    // 長さが違う場合
    if expected.len() != actual.len() && first_mismatch.is_none() {
        first_mismatch = Some(len);
    }

    let rms = if len > 0 { (sum_sq / len as f64).sqrt() as f32 } else { 0.0 };

    GoldenTestResult {
        passed: max_dev <= tolerance && expected.len() == actual.len(),
        max_deviation: max_dev,
        rms_deviation: rms,
        sample_count: len,
        first_mismatch_at: first_mismatch,
    }
}

/// Golden test: 同じプリセットを2回レンダリングして決定性を検証
pub fn verify_determinism(
    preset: &Preset,
    sample_rate: u32,
    num_samples: usize,
) -> Result<GoldenTestResult, String> {
    // テスト用入力: サイン波
    let input: Vec<f32> = (0..num_samples)
        .map(|i| (i as f32 * 440.0 * 2.0 * std::f32::consts::PI / sample_rate as f32).sin() * 0.5)
        .collect();

    let render1 = render_preset(preset, sample_rate, num_samples, &input)?;
    let render2 = render_preset(preset, sample_rate, num_samples, &input)?;

    Ok(compare_audio(&render1, &render2, 0.0)) // 完全一致
}

/// Golden reference の保存
pub fn save_golden_reference(
    output: &[f32],
    path: &std::path::Path,
) -> Result<(), String> {
    let json = serde_json::to_string(output)
        .map_err(|e| format!("Failed to serialize golden reference: {}", e))?;
    std::fs::write(path, json)
        .map_err(|e| format!("Failed to write golden reference: {}", e))
}

/// Golden reference の読み込みと比較
pub fn verify_against_golden(
    actual: &[f32],
    golden_path: &std::path::Path,
    tolerance: f32,
) -> Result<GoldenTestResult, String> {
    let json = std::fs::read_to_string(golden_path)
        .map_err(|e| format!("Failed to read golden reference: {}", e))?;
    let expected: Vec<f32> = serde_json::from_str(&json)
        .map_err(|e| format!("Failed to parse golden reference: {}", e))?;

    Ok(compare_audio(&expected, actual, tolerance))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::IrOp;
    use crate::compile::CompiledGraph;

    #[test]
    fn test_deterministic_render() {
        let program = CompiledGraph {
            ops: vec![
                IrOp::ReadInput { channel: 0 },
                IrOp::MulConst(0.5),
                IrOp::WriteOutput { channel: 0 },
            ],
            buffer_count: 0,
            state_count: 0,
        };

        let input: Vec<f32> = (0..1024).map(|i| (i as f32 * 0.01).sin()).collect();

        let out1 = render_program(program.clone(), 44100, 1024, &input);
        let out2 = render_program(program, 44100, 1024, &input);

        let result = compare_audio(&out1, &out2, 0.0);
        assert!(result.passed, "Deterministic render failed: max_dev={}", result.max_deviation);
    }

    #[test]
    fn test_gain_golden() {
        let program = CompiledGraph {
            ops: vec![
                IrOp::ReadInput { channel: 0 },
                IrOp::MulConst(0.5),
                IrOp::WriteOutput { channel: 0 },
            ],
            buffer_count: 0,
            state_count: 0,
        };

        let input: Vec<f32> = vec![1.0, 0.5, -1.0, 0.0];
        let output = render_program(program, 44100, 4, &input);
        let expected = vec![0.5, 0.25, -0.5, 0.0];

        let result = compare_audio(&expected, &output, 1e-6);
        assert!(result.passed, "Golden test failed: max_dev={}", result.max_deviation);
    }

    #[test]
    fn test_mismatch_detected() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 4.0];
        let result = compare_audio(&a, &b, 0.01);
        assert!(!result.passed);
        assert_eq!(result.first_mismatch_at, Some(2));
    }
}
