//! Forensic Telemetry Bridge.
//!
//! Streams real-time "Truth" from the Audio Engine to the UI without blocking.
//! "Reality is a stream of events, not a static state."

pub use dirtydata_core::{TelemetryEvent, TelemetryPacket, Timestamp};
use rtrb::{Producer, Consumer, RingBuffer};

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
