pub mod ir;
pub mod graph;
pub mod validate;
pub mod compile;
pub mod engine;
pub mod stdlib;
pub mod polyphony;
pub mod registry;
pub mod scripting;
pub mod preset;
pub mod param_spec;
pub mod param_graph;
pub mod hostile;
pub mod golden;
pub mod codegen;
pub mod diff;
pub mod live;
pub mod provenance;
pub mod undo;

/// オーディオバッファへのアクセスを抽象化するトレイト
/// これにより、nih_plugのバッファや自前のVecなど、バックエンドの実装を隠蔽できる
pub trait AudioBuffer {
    /// チャンネル数
    fn channels(&self) -> usize;
    /// サンプル数 (ブロックサイズ)
    fn samples(&self) -> usize;
    
    /// 指定チャンネルの不変スライスを取得
    fn channel(&mut self, ch: usize) -> &[f32];
    
    /// 指定チャンネルの可変スライスを取得
    fn channel_mut(&mut self, ch: usize) -> &mut [f32];
}

/// オーディオ処理を行う全てのユニットが実装すべきトレイト
pub trait AudioProcessor: Send + Sync {
    /// 準備処理 (サンプルレート変更時などに呼ばれる)
    fn prepare(&mut self, context: &ProcessContext);

    /// ブロックごとの処理
    /// ジェネリクスを使うことで、静的ディスパッチによる最適化を期待する
    fn process<B: AudioBuffer>(&mut self, buffer: &mut B);
    
    /// パラメータの変更 (IDベースの簡易版)
    fn set_parameter(&mut self, _id: u32, _value: f32) {}
    
    /// レイテンシ（サンプル数）を返す
    fn latency(&self) -> u32 { 0 }
}

/// 処理コンテキスト
pub struct ProcessContext {
    pub sample_rate: f64,
    pub max_block_size: usize,
    pub num_channels: usize,
}

impl ProcessContext {
    pub fn new(sample_rate: f64, max_block_size: usize, num_channels: usize) -> Self {
        Self { sample_rate, max_block_size, num_channels }
    }
}
