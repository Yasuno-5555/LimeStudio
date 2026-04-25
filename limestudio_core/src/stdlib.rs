use crate::ir::{IrOp, ParamRef, BufferId, StateId};
use serde::{Serialize, Deserialize};

/// 標準ライブラリのノード定義
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StdlibNode {
    /// ゲイン調整 (1 In, 1 Out)
    Gain { amount: ParamRef },
    /// 2つの信号のミックス (2 In, 1 Out)
    Mix { ratio: ParamRef },
    /// 1つの信号を2つに分配 (1 In, 2 Out)
    Split,
    /// パンニング (1 In, 2 Out)
    Pan { position: ParamRef },
    /// シンプルなオシレータ (0 In, 1 Out)
    Oscillator { freq: ParamRef, wave: Waveform },
    /// ADSR エンベロープ (0 In, 1 Out)
    Adsr { attack: ParamRef, decay: ParamRef, sustain: ParamRef, release: ParamRef, gate: ParamRef },
    /// 2つの信号の乗算 (2 In, 1 Out)
    Multiply,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Waveform {
    Sine,
    Saw,
    Square,
    Triangle,
}

impl StdlibNode {
    /// 入力ポート情報を返す
    pub fn input_ports(&self) -> Vec<crate::graph::PortInfo> {
        use crate::graph::{PortInfo, PortType};
        match self {
            StdlibNode::Gain { .. } => vec![PortInfo { name: "in".into(), port_type: PortType::AudioMono }],
            StdlibNode::Mix { .. } => vec![
                PortInfo { name: "in1".into(), port_type: PortType::AudioMono },
                PortInfo { name: "in2".into(), port_type: PortType::AudioMono },
            ],
            StdlibNode::Split => vec![PortInfo { name: "in".into(), port_type: PortType::AudioMono }],
            StdlibNode::Pan { .. } => vec![PortInfo { name: "in".into(), port_type: PortType::AudioMono }],
            StdlibNode::Oscillator { .. } => vec![],
            StdlibNode::Adsr { .. } => vec![],
            StdlibNode::Multiply => vec![
                PortInfo { name: "in1".into(), port_type: PortType::AudioMono },
                PortInfo { name: "in2".into(), port_type: PortType::AudioMono },
            ],
        }
    }

    /// 出力ポート情報を返す
    pub fn output_ports(&self) -> Vec<crate::graph::PortInfo> {
        use crate::graph::{PortInfo, PortType};
        match self {
            StdlibNode::Gain { .. } => vec![PortInfo { name: "out".into(), port_type: PortType::AudioMono }],
            StdlibNode::Mix { .. } => vec![PortInfo { name: "out".into(), port_type: PortType::AudioMono }],
            StdlibNode::Split => vec![
                PortInfo { name: "out1".into(), port_type: PortType::AudioMono },
                PortInfo { name: "out2".into(), port_type: PortType::AudioMono },
            ],
            StdlibNode::Pan { .. } => vec![
                PortInfo { name: "left".into(), port_type: PortType::AudioMono },
                PortInfo { name: "right".into(), port_type: PortType::AudioMono },
            ],
            StdlibNode::Oscillator { .. } => vec![PortInfo { name: "out".into(), port_type: PortType::AudioMono }],
            StdlibNode::Adsr { .. } => vec![PortInfo { name: "out".into(), port_type: PortType::AudioMono }],
            StdlibNode::Multiply => vec![PortInfo { name: "out".into(), port_type: PortType::AudioMono }],
        }
    }

    /// ノードを IR 命令列に変換する
    /// output_ids: 結果を書き込むバッファ ID のリスト
    /// inputs: 入力バッファ ID のリスト
    pub fn compile(&self, output_ids: &[BufferId], inputs: &[BufferId]) -> Vec<IrOp> {
        let mut ops = Vec::new();
        match self {
            StdlibNode::Gain { amount } => {
                if let (Some(&input), Some(&output)) = (inputs.first(), output_ids.first()) {
                    ops.push(IrOp::LoadBuffer(input));
                    match amount {
                        ParamRef::Const(v) => ops.push(IrOp::MulConst(*v)),
                        ParamRef::Param(id) => {
                            ops.push(IrOp::LoadParam(*id));
                            ops.push(IrOp::Mul);
                        }
                    }
                    ops.push(IrOp::StoreBuffer(output));
                }
            }
            StdlibNode::Mix { ratio } => {
                if inputs.len() >= 2 && !output_ids.is_empty() {
                    let in1 = inputs[0];
                    let in2 = inputs[1];
                    let output = output_ids[0];
                    
                    // (in1 * (1 - ratio)) + (in2 * ratio)
                    ops.push(IrOp::LoadBuffer(in1));
                    match ratio {
                        ParamRef::Const(v) => {
                            ops.push(IrOp::MulConst(1.0 - *v));
                        }
                        ParamRef::Param(id) => {
                            ops.push(IrOp::LoadParam(*id));
                            ops.push(IrOp::LoadConst(1.0));
                            ops.push(IrOp::Sub); // 1.0 - ratio
                            ops.push(IrOp::Mul);
                        }
                    }
                    
                    ops.push(IrOp::LoadBuffer(in2));
                    match ratio {
                        ParamRef::Const(v) => ops.push(IrOp::MulConst(*v)),
                        ParamRef::Param(id) => {
                            ops.push(IrOp::LoadParam(*id));
                            ops.push(IrOp::Mul);
                        }
                    }
                    
                    ops.push(IrOp::Add);
                    ops.push(IrOp::StoreBuffer(output));
                }
            }
            StdlibNode::Split => {
                if let Some(&input) = inputs.first() {
                    for &output in output_ids {
                        ops.push(IrOp::LoadBuffer(input));
                        ops.push(IrOp::StoreBuffer(output));
                    }
                }
            }
            StdlibNode::Pan { position } => {
                if let (Some(&input), Some(&left), Some(&right)) = (inputs.first(), output_ids.get(0), output_ids.get(1)) {
                    // Left
                    ops.push(IrOp::LoadBuffer(input));
                    match position {
                        ParamRef::Const(v) => ops.push(IrOp::MulConst(1.0 - *v)),
                        ParamRef::Param(id) => {
                            ops.push(IrOp::LoadParam(*id));
                            ops.push(IrOp::LoadConst(1.0));
                            ops.push(IrOp::Sub);
                            ops.push(IrOp::Mul);
                        }
                    }
                    ops.push(IrOp::StoreBuffer(left));

                    // Right
                    ops.push(IrOp::LoadBuffer(input));
                    match position {
                        ParamRef::Const(v) => ops.push(IrOp::MulConst(*v)),
                        ParamRef::Param(id) => {
                            ops.push(IrOp::LoadParam(*id));
                            ops.push(IrOp::Mul);
                        }
                    }
                    ops.push(IrOp::StoreBuffer(right));
                }
            }
            StdlibNode::Oscillator { freq, wave } => {
                if let Some(&output) = output_ids.first() {
                    let phase_state = StateId(0); // TODO: 状態スロットの動的割り当て
                    
                    // 1. 現在のフェーズを読み込む
                    ops.push(IrOp::LoadConst(0.0));
                    ops.push(IrOp::Delay { samples: 1, state_id: phase_state });
                    
                    // 2. フェーズを更新: phase += freq / sr
                    match freq {
                        ParamRef::Const(f) => ops.push(IrOp::LoadConst(*f)),
                        ParamRef::Param(id) => ops.push(IrOp::LoadParam(*id)),
                    }
                    ops.push(IrOp::LoadSampleRate);
                    ops.push(IrOp::Div);
                    ops.push(IrOp::Add);
                    
                    // 3. フェーズを 0.0 - 1.0 に保つ (簡易的に)
                    // TODO: Modulo プリミティブがないので、とりあえずそのまま。
                    // f32::sin は大きい値でも動くが精度が落ちる。
                    
                    // 4. 次回のために保存
                    ops.push(IrOp::StoreBuffer(output)); // スタックトップを一旦バッファへ（Brutalなスタック操作のため）
                    ops.push(IrOp::LoadBuffer(output));
                    ops.push(IrOp::Delay { samples: 0, state_id: phase_state }); // samples:0 で現在の値を保存
                    
                    // 5. 波形生成
                    ops.push(IrOp::LoadBuffer(output));
                    match wave {
                        Waveform::Sine => {
                            ops.push(IrOp::LoadConst(2.0f32 * std::f32::consts::PI));
                            ops.push(IrOp::Mul);
                            ops.push(IrOp::Sin);
                        }
                        _ => {
                            // TODO: 他の波形
                        }
                    }
                    ops.push(IrOp::StoreBuffer(output));
                }
            }
            StdlibNode::Adsr { gate, .. } => {
                if let Some(&output) = output_ids.first() {
                    // TODO: Level 1.5 では ADSR は複雑すぎるので一旦ダミー（Gate直結）
                    // ユーザーが「Polyphony を優先しろ」と言ったので、そちらに集中する。
                    match gate {
                        ParamRef::Const(v) => ops.push(IrOp::LoadConst(*v)),
                        ParamRef::Param(id) => ops.push(IrOp::LoadParam(*id)),
                    }
                    ops.push(IrOp::StoreBuffer(output));
                }
            }
            StdlibNode::Multiply => {
                if inputs.len() >= 2 && !output_ids.is_empty() {
                    ops.push(IrOp::LoadBuffer(inputs[0]));
                    ops.push(IrOp::LoadBuffer(inputs[1]));
                    ops.push(IrOp::Mul);
                    ops.push(IrOp::StoreBuffer(output_ids[0]));
                }
            }
        }
        ops
    }
}
