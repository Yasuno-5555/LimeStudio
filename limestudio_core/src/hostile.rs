//! Hostile Validation — limestudio validate --hostile
//!
//! denormal検知だけじゃ弱い。
//! JUCEはここ弱い。ここは本当に勝てる。

use crate::ir::{IrOp, BufferId};
use crate::compile::CompiledGraph;
use crate::graph::AudioGraph;

// ─── Report ────────────────────────────────────────────────

#[derive(Debug)]
pub struct ValidationReport {
    pub checks: Vec<ValidationCheck>,
}

#[derive(Debug)]
pub struct ValidationCheck {
    pub name: &'static str,
    pub severity: CheckSeverity,
    pub result: CheckResult,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckSeverity {
    /// ブロッカー。このまま出荷すると壊れる
    Critical,
    /// 高確率で問題になる
    Error,
    /// 潜在的な問題
    Warning,
    /// 情報のみ
    Info,
}

#[derive(Debug)]
pub enum CheckResult {
    Pass,
    Fail(String),
    /// 複数の問題を報告
    Issues(Vec<String>),
}

impl ValidationReport {
    pub fn has_critical(&self) -> bool {
        self.checks.iter().any(|c|
            c.severity == CheckSeverity::Critical && !matches!(c.result, CheckResult::Pass)
        )
    }

    pub fn has_errors(&self) -> bool {
        self.checks.iter().any(|c|
            (c.severity == CheckSeverity::Critical || c.severity == CheckSeverity::Error)
            && !matches!(c.result, CheckResult::Pass)
        )
    }

    pub fn print_report(&self) {
        println!("═══ LimeStudio Hostile Validation Report ═══");
        for check in &self.checks {
            let icon = match (&check.result, check.severity) {
                (CheckResult::Pass, _) => "✓",
                (_, CheckSeverity::Critical) => "✗✗",
                (_, CheckSeverity::Error) => "✗",
                (_, CheckSeverity::Warning) => "⚠",
                (_, CheckSeverity::Info) => "ℹ",
            };
            let sev = match check.severity {
                CheckSeverity::Critical => "CRITICAL",
                CheckSeverity::Error => "ERROR",
                CheckSeverity::Warning => "WARN",
                CheckSeverity::Info => "INFO",
            };
            match &check.result {
                CheckResult::Pass => {
                    println!("  {} [{}] {}", icon, sev, check.name);
                }
                CheckResult::Fail(msg) => {
                    println!("  {} [{}] {}: {}", icon, sev, check.name, msg);
                }
                CheckResult::Issues(msgs) => {
                    println!("  {} [{}] {} ({} issues)", icon, sev, check.name, msgs.len());
                    for msg in msgs {
                        println!("      → {}", msg);
                    }
                }
            }
        }
        println!("═════════════════════════════════════════════");
    }
}

// ─── Validator ─────────────────────────────────────────────

/// Hostile Validation: 全チェックを実行
pub fn validate_hostile(graph: &AudioGraph, program: &CompiledGraph) -> ValidationReport {
    let mut checks = Vec::new();

    checks.push(check_denormals(program));
    checks.push(check_nan_propagation(program));
    checks.push(check_dc_offset(program));
    checks.push(check_smoothing_missing(program));
    checks.push(check_zipper_noise(program));
    checks.push(check_feedback_latency(graph, program));
    checks.push(check_division_by_zero(program));
    checks.push(check_stack_overflow(program));
    checks.push(check_latency_report(graph, program));
    checks.push(check_unused_buffers(program));

    ValidationReport { checks }
}

// ─── Individual Checks ────────────────────────────────────

/// Denormal検知: 非常に小さい定数がフィルタ係数になりうるか
fn check_denormals(program: &CompiledGraph) -> ValidationCheck {
    let mut issues = Vec::new();

    for (i, op) in program.ops.iter().enumerate() {
        match op {
            IrOp::LoadConst(v) => {
                if *v != 0.0 && v.abs() < 1e-30 {
                    issues.push(format!("Op[{}]: LoadConst({:e}) is denormal-range", i, v));
                }
            }
            IrOp::MulConst(v) => {
                if *v != 0.0 && v.abs() < 1e-30 {
                    issues.push(format!("Op[{}]: MulConst({:e}) is denormal-range", i, v));
                }
            }
            IrOp::AddConst(v) => {
                if *v != 0.0 && v.abs() < 1e-30 {
                    issues.push(format!("Op[{}]: AddConst({:e}) is denormal-range", i, v));
                }
            }
            _ => {}
        }
    }

    ValidationCheck {
        name: "Denormal Detection",
        severity: CheckSeverity::Warning,
        result: if issues.is_empty() { CheckResult::Pass } else { CheckResult::Issues(issues) },
    }
}

/// NaN伝播: Sqrt(negative) や 0/0 のパス検出
fn check_nan_propagation(program: &CompiledGraph) -> ValidationCheck {
    let mut issues = Vec::new();

    for (i, op) in program.ops.iter().enumerate() {
        match op {
            // Sqrt は負の入力でNaNを生む
            IrOp::Sqrt => {
                // 直前にClampがない場合は警告
                if i > 0 {
                    if !matches!(program.ops[i - 1], IrOp::Clamp { min, .. } if min >= 0.0) {
                        issues.push(format!("Op[{}]: Sqrt without preceding Clamp(min>=0) — potential NaN", i));
                    }
                }
            }
            // Div の 0除算 (別途checkもあるが、NaN観点からも)
            IrOp::Div => {
                issues.push(format!("Op[{}]: Div present — ensure divisor cannot be zero at runtime", i));
            }
            _ => {}
        }
    }

    ValidationCheck {
        name: "NaN Propagation Analysis",
        severity: CheckSeverity::Error,
        result: if issues.is_empty() { CheckResult::Pass } else { CheckResult::Issues(issues) },
    }
}

/// DC Offset: 出力にAddConstがあり、それが打ち消されていないか
fn check_dc_offset(program: &CompiledGraph) -> ValidationCheck {
    let mut issues = Vec::new();
    let mut has_add_const = false;

    for (i, op) in program.ops.iter().enumerate() {
        if let IrOp::AddConst(v) = op {
            if v.abs() > 0.001 {
                has_add_const = true;
                issues.push(format!("Op[{}]: AddConst({}) may introduce DC offset", i, v));
            }
        }
    }

    // 出力直前にDCフィルタがない場合のみ警告
    if has_add_const {
        ValidationCheck {
            name: "DC Offset Detection",
            severity: CheckSeverity::Warning,
            result: CheckResult::Issues(issues),
        }
    } else {
        ValidationCheck {
            name: "DC Offset Detection",
            severity: CheckSeverity::Warning,
            result: CheckResult::Pass,
        }
    }
}

/// スムージング欠如: LoadParamの後にスムージング的なフィルタリングがない
fn check_smoothing_missing(program: &CompiledGraph) -> ValidationCheck {
    let mut issues = Vec::new();

    for (i, op) in program.ops.iter().enumerate() {
        if let IrOp::LoadParam(id) = op {
            // LoadParamの直後にMulやAddで使われている場合
            // = audio-rate で直接掛け算されている = ジッパーノイズの可能性
            if i + 1 < program.ops.len() {
                match &program.ops[i + 1] {
                    IrOp::Mul | IrOp::MulConst(_) | IrOp::Add | IrOp::AddConst(_) => {
                        issues.push(format!(
                            "Op[{}]: LoadParam({}) directly used in arithmetic at Op[{}] — no smoothing detected",
                            i, id, i + 1
                        ));
                    }
                    _ => {}
                }
            }
        }
    }

    ValidationCheck {
        name: "Parameter Smoothing Missing",
        severity: CheckSeverity::Warning,
        result: if issues.is_empty() { CheckResult::Pass } else { CheckResult::Issues(issues) },
    }
}

/// Zipper Noise: LoadParamが直接audio pathにある
fn check_zipper_noise(program: &CompiledGraph) -> ValidationCheck {
    let mut param_used_in_audio = 0;

    for op in &program.ops {
        if matches!(op, IrOp::LoadParam(_)) {
            param_used_in_audio += 1;
        }
    }

    ValidationCheck {
        name: "Zipper Noise Risk",
        severity: CheckSeverity::Info,
        result: if param_used_in_audio == 0 {
            CheckResult::Pass
        } else {
            CheckResult::Fail(format!(
                "{} parameter reads in audio path — ensure all are smoothed at host level",
                param_used_in_audio
            ))
        },
    }
}

/// フィードバックレイテンシ: Delayの使用状況
fn check_feedback_latency(graph: &AudioGraph, program: &CompiledGraph) -> ValidationCheck {
    let mut delay_count = 0;
    let mut total_delay_samples = 0u32;

    for op in &program.ops {
        if let IrOp::Delay { samples, .. } = op {
            delay_count += 1;
            total_delay_samples = total_delay_samples.saturating_add(*samples);
        }
    }

    ValidationCheck {
        name: "Feedback Latency",
        severity: CheckSeverity::Info,
        result: if delay_count == 0 {
            CheckResult::Pass
        } else {
            CheckResult::Fail(format!(
                "{} delay nodes, total {} samples ({:.1}ms at 44100Hz)",
                delay_count,
                total_delay_samples,
                total_delay_samples as f64 / 44.1
            ))
        },
    }
}

/// 0除算: Divの存在チェック
fn check_division_by_zero(program: &CompiledGraph) -> ValidationCheck {
    let mut issues = Vec::new();

    for (i, op) in program.ops.iter().enumerate() {
        if matches!(op, IrOp::Div) {
            issues.push(format!("Op[{}]: Division — ensure divisor is guarded", i));
        }
    }

    ValidationCheck {
        name: "Division by Zero Guard",
        severity: CheckSeverity::Error,
        result: if issues.is_empty() { CheckResult::Pass } else { CheckResult::Issues(issues) },
    }
}

/// スタックオーバーフロー: スタック深さの静的解析
fn check_stack_overflow(program: &CompiledGraph) -> ValidationCheck {
    let mut depth: i32 = 0;
    let mut max_depth: i32 = 0;

    for (i, op) in program.ops.iter().enumerate() {
        match op {
            // Push系
            IrOp::LoadConst(_) | IrOp::LoadParam(_) | IrOp::LoadBuffer(_)
            | IrOp::ReadInput { .. } | IrOp::LoadSampleRate => {
                depth += 1;
            }
            // Pop-Push系 (net = 0)
            IrOp::MulConst(_) | IrOp::AddConst(_) | IrOp::Clamp { .. }
            | IrOp::Abs | IrOp::Sqrt | IrOp::Neg | IrOp::Sin | IrOp::Cos
            | IrOp::Delay { .. } => {}
            // Pop-Pop-Push系 (net = -1)
            IrOp::Add | IrOp::Mul | IrOp::Sub | IrOp::Div => {
                depth -= 1;
            }
            // Store系 (peek, no pop)
            IrOp::StoreBuffer(_) => {}
            // Pop系
            IrOp::WriteOutput { .. } => {
                depth -= 1;
            }
            // Buffer-to-buffer (no stack effect)
            IrOp::CopyBuffer(_, _) | IrOp::AddBuffer(_, _) => {}
        }
        if depth > max_depth { max_depth = depth; }
    }

    let stack_capacity = 64; // SampleStack::data size
    if max_depth > stack_capacity {
        ValidationCheck {
            name: "Stack Overflow Analysis",
            severity: CheckSeverity::Critical,
            result: CheckResult::Fail(format!(
                "Max stack depth {} exceeds capacity {}",
                max_depth, stack_capacity
            )),
        }
    } else {
        ValidationCheck {
            name: "Stack Overflow Analysis",
            severity: CheckSeverity::Critical,
            result: CheckResult::Pass,
        }
    }
}

/// レイテンシレポート: 累積レイテンシの計算
fn check_latency_report(_graph: &AudioGraph, program: &CompiledGraph) -> ValidationCheck {
    let mut total_latency: u32 = 0;

    for op in &program.ops {
        if let IrOp::Delay { samples, .. } = op {
            if *samples > 0 {
                total_latency = total_latency.saturating_add(*samples);
            }
        }
    }

    ValidationCheck {
        name: "Latency Report",
        severity: CheckSeverity::Info,
        result: CheckResult::Fail(format!(
            "Total plugin latency: {} samples ({:.2}ms at 44100Hz, {:.2}ms at 48000Hz)",
            total_latency,
            total_latency as f64 / 44.1,
            total_latency as f64 / 48.0
        )),
    }
}

/// 未使用バッファ検出
fn check_unused_buffers(program: &CompiledGraph) -> ValidationCheck {
    let mut written: std::collections::HashSet<u32> = std::collections::HashSet::new();
    let mut read: std::collections::HashSet<u32> = std::collections::HashSet::new();

    for op in &program.ops {
        match op {
            IrOp::StoreBuffer(id) => { written.insert(id.0); }
            IrOp::LoadBuffer(id) => { read.insert(id.0); }
            IrOp::CopyBuffer(src, dst) => { read.insert(src.0); written.insert(dst.0); }
            IrOp::AddBuffer(src, dst) => { read.insert(src.0); written.insert(dst.0); }
            _ => {}
        }
    }

    let written_never_read: Vec<u32> = written.difference(&read).copied().collect();

    if written_never_read.is_empty() {
        ValidationCheck {
            name: "Unused Buffer Detection",
            severity: CheckSeverity::Warning,
            result: CheckResult::Pass,
        }
    } else {
        ValidationCheck {
            name: "Unused Buffer Detection",
            severity: CheckSeverity::Warning,
            result: CheckResult::Fail(format!(
                "Buffers written but never read: {:?}",
                written_never_read
            )),
        }
    }
}

// ─── Tests ─────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::*;
    use crate::graph::{AudioGraph, GraphNode};

    fn simple_program() -> (AudioGraph, CompiledGraph) {
        let mut graph = AudioGraph::new();
        graph.add_node(GraphNode::Input { channel: 0 });
        graph.add_node(GraphNode::Output { channel: 0 });

        let program = CompiledGraph {
            ops: vec![
                IrOp::ReadInput { channel: 0 },
                IrOp::MulConst(0.5),
                IrOp::WriteOutput { channel: 0 },
            ],
            buffer_count: 0,
            state_count: 0,
        };
        (graph, program)
    }

    #[test]
    fn test_clean_program_passes() {
        let (graph, program) = simple_program();
        let report = validate_hostile(&graph, &program);
        assert!(!report.has_critical());
    }

    #[test]
    fn test_denormal_detected() {
        let program = CompiledGraph {
            ops: vec![IrOp::LoadConst(1e-40)],
            buffer_count: 0,
            state_count: 0,
        };
        let graph = AudioGraph::new();
        let report = validate_hostile(&graph, &program);
        let denorm = report.checks.iter().find(|c| c.name == "Denormal Detection").unwrap();
        assert!(!matches!(denorm.result, CheckResult::Pass));
    }

    #[test]
    fn test_stack_overflow_detected() {
        // 65 consecutive pushes -> overflow
        let mut ops: Vec<IrOp> = (0..65).map(|_| IrOp::LoadConst(1.0)).collect();
        let program = CompiledGraph { ops, buffer_count: 0, state_count: 0 };
        let graph = AudioGraph::new();
        let report = validate_hostile(&graph, &program);
        let stack = report.checks.iter().find(|c| c.name == "Stack Overflow Analysis").unwrap();
        assert!(matches!(stack.result, CheckResult::Fail(_)));
    }

    #[test]
    fn test_sqrt_nan_warning() {
        let program = CompiledGraph {
            ops: vec![
                IrOp::ReadInput { channel: 0 },
                IrOp::Sqrt, // no clamp before
            ],
            buffer_count: 0,
            state_count: 0,
        };
        let graph = AudioGraph::new();
        let report = validate_hostile(&graph, &program);
        let nan = report.checks.iter().find(|c| c.name == "NaN Propagation Analysis").unwrap();
        assert!(!matches!(nan.result, CheckResult::Pass));
    }
}
