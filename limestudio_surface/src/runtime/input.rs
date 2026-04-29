//! Lime Surface Input Abstraction
//!
//! Never bind directly to winit. Use normalized events.

use glam::Vec2;

#[derive(Debug, Clone, Copy, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum SurfaceEvent {
    PointerMove {
        position: Vec2,
        modifiers: Modifiers,
    },
    PointerDown {
        position: Vec2,
        button: MouseButton,
        modifiers: Modifiers,
    },
    PointerUp {
        position: Vec2,
        button: MouseButton,
        modifiers: Modifiers,
    },
    KeyInput {
        key: Key,
        pressed: bool,
        modifiers: Modifiers,
    },
    Wheel {
        delta: Vec2,
        modifiers: Modifiers,
    },
    Focus(bool),
    Resize {
        width: u32,
        height: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Shift,
    Control,
    Alt,
    Space,
    Delete,
    Enter,
    Escape,
    Tab,
    Backspace,
    Up,
    Down,
    Left,
    Right,
    Char(char),
}

/// Drag Hysteresis Standard
pub struct InteractionState {
    pub drag_origin: Option<Vec2>,
    pub accumulated_delta: Vec2,
    pub is_dragging: bool,
    pub threshold: f32, // Default: 3.0 logical pixels
}

impl InteractionState {
    pub fn new(threshold: f32) -> Self {
        Self {
            drag_origin: None,
            accumulated_delta: Vec2::ZERO,
            is_dragging: false,
            threshold,
        }
    }

    pub fn handle_event(&mut self, event: &SurfaceEvent) {
        match event {
            SurfaceEvent::PointerDown {
                position,
                button: MouseButton::Left,
                ..
            } => {
                self.drag_origin = Some(*position);
                self.accumulated_delta = Vec2::ZERO;
                self.is_dragging = false;
            }
            SurfaceEvent::PointerMove { position, .. } => {
                if let Some(origin) = self.drag_origin {
                    let delta = *position - origin;
                    self.accumulated_delta = delta;

                    if !self.is_dragging && delta.length() >= self.threshold {
                        self.is_dragging = true;
                    }
                }
            }
            SurfaceEvent::PointerUp { .. } => {
                self.drag_origin = None;
                self.is_dragging = false;
            }
            _ => {}
        }
    }
}

/// Event Priority for the Airbag
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPriority {
    Low,      // Background tasks
    Standard, // Normal move/interaction
    High,     // Click, Key, Essential UI
    Critical, // OS Resize, Adapter changes
}

#[derive(Debug, Clone, Copy)]
pub struct TimedEvent {
    pub event: SurfaceEvent,
    pub timestamp: std::time::Instant,
    pub priority: EventPriority,
}

/// Wait-Free Event Bridge (The Airbag)
///
/// OSスレッド(Producer)とLime UIスレッド(Consumer)を、
/// 非ブロッキングかつセマンティックな圧縮を伴って橋渡しする。
pub struct WaitFreeEventBridge {
    /// Discrete Track: Down/Up/Key/Resize など、「一発も落とせない」イベント用。
    discrete_queue: rtrb::Producer<TimedEvent>,
    /// Continuous Track: PointerMove など、「最新値が正義」のイベント用。
    /// Packed (f32 x, f32 y, u64 timestamp_micros)
    latest_move: std::sync::Arc<MoveState>,
}

pub struct MoveState {
    pub packed: std::sync::atomic::AtomicU64,
    pub timestamp: std::sync::atomic::AtomicU64,
}

impl WaitFreeEventBridge {
    pub fn new(capacity: usize) -> (Self, rtrb::Consumer<TimedEvent>, std::sync::Arc<MoveState>) {
        let (p, c) = rtrb::RingBuffer::new(capacity);
        let move_state = std::sync::Arc::new(MoveState {
            packed: std::sync::atomic::AtomicU64::new(0),
            timestamp: std::sync::atomic::AtomicU64::new(0),
        });
        (
            Self {
                discrete_queue: p,
                latest_move: move_state.clone(),
            },
            c,
            move_state,
        )
    }

    pub fn push(&mut self, event: SurfaceEvent) {
        let timestamp = std::time::Instant::now();
        let priority = match event {
            SurfaceEvent::PointerMove { .. } => EventPriority::Standard,
            SurfaceEvent::Resize { .. } | SurfaceEvent::Focus(_) => EventPriority::Critical,
            _ => EventPriority::High,
        };

        match event {
            SurfaceEvent::PointerMove { position, .. } => {
                // Continuous Track: 最新値で上書き（セマンティック圧縮）
                let x = position.x.to_bits();
                let y = position.y.to_bits();
                let packed = (x as u64) << 32 | (y as u64);

                self.latest_move
                    .packed
                    .store(packed, std::sync::atomic::Ordering::Release);
                self.latest_move.timestamp.store(
                    timestamp.elapsed().as_micros() as u64,
                    std::sync::atomic::Ordering::Release,
                );

                // Moveもキューに積むが、満杯なら捨てる（最新値が atomic にあるため）
                let _ = self.discrete_queue.push(TimedEvent {
                    event,
                    timestamp,
                    priority,
                });
            }
            _ => {
                // Discrete Track: 満杯の場合は古いMoveイベントを探して捨てるなどのパージ戦略が必要。
                // 現状はシンプルにプッシュを試みる。
                if self.discrete_queue.is_full() {
                    // TODO: 緊急パージロジックの実装（古いPointerMoveを捨てる）
                }
                let _ = self.discrete_queue.push(TimedEvent {
                    event,
                    timestamp,
                    priority,
                });
            }
        }
    }
}
