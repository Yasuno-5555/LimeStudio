//! Diagnostic Vascular System - 「Realityを安全に観測するための血管系」
//! 
//! SPSC (Single-Producer, Single-Consumer) を用い、
//! オーディオスレッドからUIスレッドへ診断データをWait-Freeに転送します。

use rtrb::{Producer, Consumer, RingBuffer};

/// Spectrum表示用のビン数（対数スケールバケット）
pub const SPECTRUM_BINS: usize = 128;
/// Waveform表示用のサンプル数（デシメーション済み）
pub const WAVEFORM_SAMPLES: usize = 256;

/// 観測データの種類。
/// RTスレッドでのアロケーションを避けるため、データは固定長配列で保持します。
#[allow(clippy::large_enum_variant)]
pub enum ObservationEvent {
    /// チャンネルごとのピーク値
    Peak { left: f32, right: f32 },
    /// 波形データ (Time Domain)
    Waveform([f32; WAVEFORM_SAMPLES]),
    /// スペクトラムデータ (Frequency Domain - Magnitude Only)
    Spectrum([f32; SPECTRUM_BINS]),
    /// CPU負荷やスレッドのジッター状況
    PerfStats { cpu_usage: f32, jitter_ms: f32 },
    /// 特定ノードのデバッグメッセージ
    Trace { node_id: &'static str, message: String },
}

/// オーディオスレッド側：データを「血管」に流し込む
pub struct ObservationProducer {
    producer: Producer<ObservationEvent>,
}

/// UI/診断側：データを「血管」から吸い上げる
pub struct ObservationConsumer {
    consumer: Consumer<ObservationEvent>,
}

impl ObservationProducer {
    /// データを送信。キューが満杯の場合はドロップします。
    #[inline(always)]
    pub fn emit(&mut self, event: ObservationEvent) {
        let _ = self.producer.push(event);
    }
}

impl ObservationConsumer {
    /// 溜まっている観測データをすべて読み出します。
    pub fn drain(&mut self) -> Vec<ObservationEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.consumer.pop() {
            events.push(event);
        }
        events
    }

    /// ダミーのコンシューマーを作成します（何も流れてきません）。
    pub fn dummy() -> Self {
        let (_, cons) = RingBuffer::new(1);
        Self { consumer: cons }
    }
}

/// 新しい観測パイプラインを作成します。
pub fn create_pipeline(capacity: usize) -> (ObservationProducer, ObservationConsumer) {
    let (prod, cons) = RingBuffer::new(capacity);
    (
        ObservationProducer { producer: prod },
        ObservationConsumer { consumer: cons },
    )
}

/// 信号のピーク値を監視するためのヘルパー。
pub struct PeakMonitor {
    left_max: f32,
    right_max: f32,
    count: usize,
    interval: usize,
}

impl PeakMonitor {
    pub fn new(interval: usize) -> Self {
        Self {
            left_max: 0.0,
            right_max: 0.0,
            count: 0,
            interval,
        }
    }

    #[inline(always)]
    pub fn process(&mut self, left: f32, right: f32, producer: &mut ObservationProducer) {
        self.left_max = self.left_max.max(left.abs());
        self.right_max = self.right_max.max(right.abs());
        self.count += 1;
        
        if self.count >= self.interval {
            producer.emit(ObservationEvent::Peak {
                left: self.left_max,
                right: self.right_max,
            });
            self.left_max = 0.0;
            self.right_max = 0.0;
            self.count = 0;
        }
    }
}

/// 波形データをデシメーションしながら収集するヘルパー。
pub struct WaveformMonitor {
    buffer: [f32; WAVEFORM_SAMPLES],
    write_pos: usize,
    decimation_count: usize,
    decimation_interval: usize,
}

impl WaveformMonitor {
    pub fn new(decimation_interval: usize) -> Self {
        Self {
            buffer: [0.0; WAVEFORM_SAMPLES],
            write_pos: 0,
            decimation_count: 0,
            decimation_interval,
        }
    }

    #[inline(always)]
    pub fn process(&mut self, sample: f32, producer: &mut ObservationProducer) {
        self.decimation_count += 1;
        if self.decimation_count >= self.decimation_interval {
            self.buffer[self.write_pos] = sample;
            self.write_pos += 1;
            self.decimation_count = 0;

            if self.write_pos >= WAVEFORM_SAMPLES {
                producer.emit(ObservationEvent::Waveform(self.buffer));
                self.write_pos = 0;
            }
        }
    }
}
