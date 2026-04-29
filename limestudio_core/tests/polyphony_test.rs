use dirtydata_core::ir::Graph;
use limestudio_core::engine::{VoiceEvent, VoiceManager};
use limestudio_core::PatchEvent;
use rtrb::RingBuffer;

#[test]
fn test_voice_allocation() {
    let graph = Graph::new(); // Empty graph
    let (_prod, cons) = RingBuffer::<PatchEvent>::new(64);
    let mut manager = VoiceManager::from_graph(&graph, cons, 4, 44100.0, None);

    // Initial state: all idle
    assert_eq!(manager.voices.len(), 4);
    assert!(manager
        .voices
        .iter()
        .all(|v| v.state == limestudio_core::engine::VoiceState::Idle));

    // Note On
    manager.handle_event(VoiceEvent::NoteOn {
        pitch: 60,
        velocity: 0.8,
    });
    let active_count = manager
        .voices
        .iter()
        .filter(|v| matches!(v.state, limestudio_core::engine::VoiceState::Active { .. }))
        .count();
    assert_eq!(active_count, 1);

    // Fill up
    manager.handle_event(VoiceEvent::NoteOn {
        pitch: 62,
        velocity: 0.8,
    });
    manager.handle_event(VoiceEvent::NoteOn {
        pitch: 64,
        velocity: 0.8,
    });
    manager.handle_event(VoiceEvent::NoteOn {
        pitch: 65,
        velocity: 0.8,
    });
    assert_eq!(
        manager
            .voices
            .iter()
            .filter(|v| matches!(v.state, limestudio_core::engine::VoiceState::Active { .. }))
            .count(),
        4
    );

    // Steal attempt
    manager.handle_event(VoiceEvent::NoteOn {
        pitch: 67,
        velocity: 0.8,
    });
    // One voice should now be stealing
    let stealing_count = manager
        .voices
        .iter()
        .filter(|v| {
            matches!(
                v.state,
                limestudio_core::engine::VoiceState::Stealing { .. }
            )
        })
        .count();
    assert_eq!(stealing_count, 1);
}
