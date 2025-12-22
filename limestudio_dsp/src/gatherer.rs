use std::collections::VecDeque;

pub struct BlockGatherer {
    // データ保持用
    input_buffer: VecDeque<f32>,
    output_buffer: VecDeque<f32>,
    
    // 設定
    window_size: usize, // FFT長 (例: 2048)
    hop_size: usize,    // ずらす幅 (例: 512)
    
    // 一時バッファ
    process_input_scratch: Vec<f32>,
    
    // アロケーション削減のための作業用バッファ (Callbackの出力受け取り用)
    process_output_scratch: Vec<f32>,
}

impl BlockGatherer {
    pub fn new(window_size: usize, hop_size: usize) -> Self {
        Self {
            input_buffer: VecDeque::with_capacity(window_size * 2),
            output_buffer: VecDeque::with_capacity(window_size * 2),
            window_size,
            hop_size,
            process_input_scratch: vec![0.0; window_size],
            process_output_scratch: vec![0.0; window_size],
        }
    }

    /// DAWからの入力を受け取り、処理可能な分だけ callback を実行し、
    /// 結果を output_slice に書き込む
    /// process_callback: 入力(window_size) -> 出力(window_size) (Add済みではなく、純粋なブロック応答)
    pub fn process_stream<F>(
        &mut self,
        input_slice: &[f32],
        output_slice: &mut [f32],
        mut process_callback: F,
    ) 
    where
        F: FnMut(&[f32], &mut [f32]),
    {
        // 1. 入力を溜める
        self.input_buffer.extend(input_slice);

        // 2. ウィンドウサイズ分溜まったら処理を回す
        while self.input_buffer.len() >= self.window_size {
            // A. 入力データの準備
            for i in 0..self.window_size {
                self.process_input_scratch[i] = self.input_buffer[i];
            }

            // B. コールバック実行
            // 出力スクラッチをクリアしてから渡すか、callback内で上書きするか
            // ここではcallbackが上書き(fill)することを期待する
            process_callback(&self.process_input_scratch, &mut self.process_output_scratch);

            // C. Output Bufferへの OLA (Overlap-Add) 加算書き込み
            // 足りなければ拡張 (倍以上確保してreallocation減らす)
            if self.output_buffer.len() < self.window_size + self.hop_size {
                self.output_buffer.resize(self.output_buffer.len() + self.window_size * 2, 0.0);
            }
            
            // OLA: Add to existing buffer
            for (i, &sample) in self.process_output_scratch.iter().enumerate() {
                if i < self.output_buffer.len() {
                    self.output_buffer[i] += sample;
                } else {
                    // resizeしてるので基本ここは通らないはずだが念のため
                    self.output_buffer.push_back(sample);
                }
            }

            // D. Hop (入力バッファを進める)
            self.input_buffer.drain(0..self.hop_size);
        }

        // 3. DAWへの出力
        // レイテンシ分（window_size - hop_size）が自動的に溜まった状態からpopされる?
        // いいえ、出力バッファは初期状態では空。
        // 最初のブロック処理が終わると、output_bufferにデータが入る。
        // OLAの性質上、前のブロックのテールが残っている。
        // ここから hop_size 分だけ取り出すのがストリーム処理の基本だが、
        // DAWからの要求量 (output_slice.len()) は任意。
        // なので、あるだけ出す。足りなければ0埋め。
        
        for sample in output_slice.iter_mut() {
            if let Some(val) = self.output_buffer.pop_front() {
                *sample = val;
            } else {
                *sample = 0.0; // Underflow / Latency
            }
        }
    }
    
    pub fn latency_samples(&self) -> u32 {
        (self.window_size - self.hop_size) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gatherer_latency() {
        let win = 16;
        let hop = 4;
        let mut gatherer = BlockGatherer::new(win, hop);
        
        // 100 samples of 1.0
        let input = vec![1.0; 100]; 
        let mut output = vec![0.0; 100];
        
        // Thru callback: copy input scratch to output scratch
        gatherer.process_stream(&input, &mut output, |src, dst| {
            dst.copy_from_slice(src); 
        });
        
        // Check Latency: first (win - hop) samples should be 0.0
        let latency = win - hop;
        for i in 0..latency {
            assert_eq!(output[i], 0.0, "Index {} should be 0.0 due to latency", i);
        }
        
        // Check Signal: after latency, it should be 1.0 (assuming perfect OLA if we did fancy windowing, 
        // but here we just copy blocks. 
        // Overlap-Add of constant 1.0 blocks with Rect window:
        // Overlap factor = win / hop = 16 / 4 = 4.
        // Result should be 4.0 if we just add them up.
        // Wait, did I normalize in the callback? No.
        // In the test callback: `dst.copy_from_slice(src)`. src is 1.0. dst becomes 1.0.
        // Convert to OLA:
        // We add 1.0 to the buffer.
        // Overlap 4x.
        // So the steady state output should be 4.0.
        
        // Let's check indices after initial transient (e.g. at index 50)
        assert_eq!(output[50], 4.0, "Steady state should be overlap count (4.0)");
    }
}
