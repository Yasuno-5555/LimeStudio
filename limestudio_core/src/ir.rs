use serde::{Serialize, Deserialize};

/// パラメータ識別子 (index-based)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct ParamId(pub u32);

/// バッファスロット識別子 (Node間通信用)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct BufferId(pub u32);

/// 状態スロット識別子 (Delay等のStateful Primitive用)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct StateId(pub u32);

/// パラメータ参照
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ParamRef {
    /// 固定値
    Const(f32),
    /// パラメータ参照
    Param(ParamId),
}

/// プリミティブ IR オペコード
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum IrOp {
    // --- Stack Manipulation ---
    /// 定数をスタックに積む
    LoadConst(f32),
    /// パラメータ値をスタックに積む (AtomicU32から取得)
    LoadParam(ParamId),
    /// バッファからスタックにロード
    LoadBuffer(BufferId),
    /// スタックトップをバッファにストア (スタックは不変)
    StoreBuffer(BufferId),

    // --- Arithmetic (Binary) ---
    /// a + b
    Add,
    /// a * b
    Mul,
    /// a * const
    MulConst(f32),
    /// a - b
    Sub,
    /// a / b
    Div,
    /// a + const
    AddConst(f32),

    // --- Arithmetic (Unary/Special) ---
    /// [min, max] にクランプ
    Clamp { min: f32, max: f32 },
    /// 絶対値
    Abs,
    /// 平方根
    Sqrt,
    /// 符号反転
    Neg,
    /// 正弦波
    Sin,
    /// 余弦波
    Cos,
    /// サンプルレートをスタックに積む
    LoadSampleRate,

    // --- Stateful ---
    /// サンプル単位のディレイ
    /// state_id は DspEngine 内のディレイラインを参照
    Delay { samples: u32, state_id: StateId },

    /// DAWからの入力をスタックに積む
    ReadInput { channel: u8 },
    /// スタックトップをDAWの出力に書く (スタックからpop)
    WriteOutput { channel: u8 },

    // --- Buffer-to-Buffer (Optimized) ---
    /// バッファ A から バッファ B へコピー
    CopyBuffer(BufferId, BufferId),
    /// バッファ A を バッファ B に加算 (B += A)
    AddBuffer(BufferId, BufferId),
}

impl std::fmt::Display for IrOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IrOp::LoadConst(v) => write!(f, "LoadConst({})", v),
            IrOp::LoadParam(id) => write!(f, "LoadParam({})", id),
            IrOp::LoadBuffer(id) => write!(f, "LoadBuffer({})", id),
            IrOp::StoreBuffer(id) => write!(f, "StoreBuffer({})", id),
            IrOp::Add => write!(f, "Add"),
            IrOp::Mul => write!(f, "Mul"),
            IrOp::MulConst(v) => write!(f, "MulConst({})", v),
            IrOp::Sub => write!(f, "Sub"),
            IrOp::Div => write!(f, "Div"),
            IrOp::AddConst(v) => write!(f, "AddConst({})", v),
            IrOp::Clamp { min, max } => write!(f, "Clamp({}, {})", min, max),
            IrOp::Abs => write!(f, "Abs"),
            IrOp::Sqrt => write!(f, "Sqrt"),
            IrOp::Neg => write!(f, "Neg"),
            IrOp::Sin => write!(f, "Sin"),
            IrOp::Cos => write!(f, "Cos"),
            IrOp::LoadSampleRate => write!(f, "LoadSampleRate"),
            IrOp::Delay { samples, state_id } => write!(f, "Delay(samples={}, state={})", samples, state_id),
            IrOp::ReadInput { channel } => write!(f, "ReadInput(ch={})", channel),
            IrOp::WriteOutput { channel } => write!(f, "WriteOutput(ch={})", channel),
            IrOp::CopyBuffer(src, dst) => write!(f, "CopyBuffer(src={}, dst={})", src, dst),
            IrOp::AddBuffer(src, dst) => write!(f, "AddBuffer(src={}, dst={})", src, dst),
        }
    }
}

impl std::fmt::Display for ParamId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "P{}", self.0)
    }
}

impl std::fmt::Display for BufferId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "B{}", self.0)
    }
}

impl std::fmt::Display for StateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "S{}", self.0)
    }
}

/// オーディオスレッドセーフな固定長スタック
/// パニックパスを避け、境界チェックを最小限にする（または上位で保証する）
pub struct SampleStack {
    data: [f32; 64],
    sp: usize,
}

impl SampleStack {
    pub fn new() -> Self {
        Self {
            data: [0.0; 64],
            sp: 0,
        }
    }

    /// スタックをクリアする (RT-safe)
    #[inline]
    pub fn clear(&mut self) {
        self.sp = 0;
    }

    /// 値をプッシュする
    /// Level 0 ではスタックオーバーフローのチェックは compile/validate 時に行う前提
    #[inline]
    pub fn push(&mut self, val: f32) {
        if self.sp < 64 {
            self.data[self.sp] = val;
            self.sp += 1;
        }
        // overflow は無視（上位で保証すべき）
    }

    /// 値をポップする
    #[inline]
    pub fn pop(&mut self) -> f32 {
        if self.sp > 0 {
            self.sp -= 1;
            self.data[self.sp]
        } else {
            0.0 // underflow 安全弁
        }
    }
    
    #[inline]
    pub fn peek(&self) -> f32 {
        if self.sp > 0 {
            self.data[self.sp - 1]
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_basic() {
        let mut stack = SampleStack::new();
        stack.push(1.0);
        stack.push(2.0);
        assert_eq!(stack.pop(), 2.0);
        assert_eq!(stack.pop(), 1.0);
        assert_eq!(stack.pop(), 0.0);
    }
}
