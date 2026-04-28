//! Forensic Telemetry Bridge.
//!
//! Streams real-time "Truth" from the Audio Engine to the UI without blocking.
//! "Reality is a stream of events, not a static state."

use dirtydata_core::types::{Timestamp, StableId};
use rtrb::{Producer, Consumer, RingBuffer};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryPacket {
    pub timestamp: Timestamp,
    pub event: TelemetryEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TelemetryEvent {
    /// Audio engine CPU usage per block.
    CpuUsage { micros: f32 },
    /// Digital clipping detected on a channel.
    ClipDetected { channel: u32, peak: f32 },
    /// NaN or Infinity detected in the signal path.
    NanDetected { node_id: Option<StableId> },
    /// A voice became active.
    VoiceActive { index: usize, pitch: u8, velocity: f32 },
    /// A voice entered release phase.
    VoiceReleased { index: usize },
    /// JIT compiler or execution error.
    JitError { message: String },
    /// Divergence detected between expected and actual state.
    DivergenceReport { node_id: StableId, confidence: f32 },
}

/// The producing end of the telemetry bridge (owned by the Audio Engine).
pub struct TelemetryProducer {
    tx: Producer<TelemetryPacket>,
}

impl TelemetryProducer {
    pub fn push(&mut self, event: TelemetryEvent) {
        let packet = TelemetryPacket {
            timestamp: Timestamp::now(),
            event,
        };
        // Use a non-blocking push. If the buffer is full, we drop the telemetry
        // rather than blocking the audio thread.
        let _ = self.tx.push(packet);
    }
    pub fn from_raw(tx: Producer<TelemetryPacket>) -> Self {
        Self { tx }
    }
}

/// The consuming end of the telemetry bridge (owned by the UI).
pub struct TelemetryConsumer {
    rx: Consumer<TelemetryPacket>,
}

impl TelemetryConsumer {
    pub fn from_raw(rx: Consumer<TelemetryPacket>) -> Self {
        Self { rx }
    }

    /// Drains all available packets from the buffer.
    pub fn drain(&mut self) -> Vec<TelemetryPacket> {
        let mut packets = Vec::new();
        while let Ok(packet) = self.rx.pop() {
            packets.push(packet);
        }
        packets
    }
}

/// Creates a new Forensic Telemetry Bridge.
pub fn create_bridge(capacity: usize) -> (TelemetryProducer, TelemetryConsumer) {
    let (tx, rx) = RingBuffer::new(capacity);
    (
        TelemetryProducer { tx },
        TelemetryConsumer { rx },
    )
}
