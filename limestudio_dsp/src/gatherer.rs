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
        // 十分なキャパシティを確保しておく (WindowSizeの数倍あれば通常は足りる)
        let initial_capacity = window_size * 4;
        Self {
            input_buffer: VecDeque::with_capacity(initial_capacity),
            output_buffer: VecDeque::with_capacity(initial_capacity),
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
    ) where
        F: FnMut(&[f32], &mut [f32]),
    {
        // 1. 入力を溜める
        self.input_buffer.extend(input_slice);

        // 2. ウィンドウサイズ分溜まったら処理を回す
        let mut hops_processed = 0;
        while self.input_buffer.len() >= self.window_size {
            // A. 入力データの準備
            for i in 0..self.window_size {
                self.process_input_scratch[i] = self.input_buffer[i];
            }

            // B. コールバック実行
            process_callback(
                &self.process_input_scratch,
                &mut self.process_output_scratch,
            );

            // C. Output Bufferへの OLA (Overlap-Add) 加算書き込み
            // 処理が重なるため、正しいオフセット位置に加算する必要がある
            let offset = hops_processed * self.hop_size;
            for (i, &sample) in self.process_output_scratch.iter().enumerate() {
                let target_idx = i + offset;
                if target_idx < self.output_buffer.len() {
                    self.output_buffer[target_idx] += sample;
                } else {
                    // 足りない場合は 0 で埋めてから追加
                    while self.output_buffer.len() < target_idx {
                        self.output_buffer.push_back(0.0);
                    }
                    self.output_buffer.push_back(sample);
                }
            }

            // D. Hop (入力バッファを進める)
            self.input_buffer.drain(0..self.hop_size);
            hops_processed += 1;
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

        // Check Transient: The first samples should reflect the OLA buildup.
        // Index 0: 1 block contributes -> 1.0
        // Index 4: 2 blocks contribute -> 2.0
        // Index 8: 3 blocks contribute -> 3.0
        // Index 12: 4 blocks contribute (Steady State) -> 4.0
        assert_eq!(output[0], 1.0, "Index 0 should be 1.0");
        assert_eq!(output[4], 2.0, "Index 4 should be 2.0");
        assert_eq!(output[8], 3.0, "Index 8 should be 3.0");
        assert_eq!(output[12], 4.0, "Index 12 should reach steady state 4.0");

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
        assert_eq!(
            output[50], 4.0,
            "Steady state should be overlap count (4.0)"
        );
    }
}
