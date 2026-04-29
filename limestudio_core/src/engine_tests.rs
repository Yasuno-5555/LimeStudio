#[cfg(test)]
mod tests {
    use crate::engine::VoiceManager;
    use crate::graph::GraphBuilder;
    use crate::PatchEvent;
    use rtrb::RingBuffer;

    #[test]
    fn test_engine_gain_processing() {
        let mut builder = GraphBuilder::new();
        let input_node = builder.input_node();
        let output_node = builder.output_node();

        // Gain node with parameter "gain"
        let gain_node = builder.add_processor(
            "Gain",
            vec![(
                "gain",
                crate::graph::ParamSource::Parameter("gain".to_string()),
            )],
        );

        builder.connect(input_node, gain_node);
        builder.connect(gain_node, output_node);

        let graph = builder.build();
        let (mut prod, cons) = RingBuffer::<PatchEvent>::new(64);
        let mut engine = VoiceManager::from_graph(&graph, cons, 1, 44100.0, None);

        // Active a voice to test gain
        engine.handle_event(crate::engine::VoiceEvent::NoteOn {
            pitch: 60,
            velocity: 1.0,
        });

        // Prepare data: 1.0 everywhere
        let in_l = vec![1.0f32; 128];
        let in_r = vec![1.0f32; 128];
        let inputs = vec![in_l.as_slice(), in_r.as_slice()];

        let mut out_l = vec![0.0f32; 128];
        let mut out_r = vec![0.0f32; 128];
        let mut outputs = vec![out_l.as_mut_slice(), out_r.as_mut_slice()];

        // 1. Initial gain is 1.0 (default in engine initialization usually, but let's set it)
        prod.push(PatchEvent::SetParameter {
            param_id: "gain".to_string(),
            value: 0.5,
        })
        .unwrap();

        engine.process(&inputs, &mut outputs, 44100.0);

        // Gain 0.5 applied to 1.0 should be 0.5
        assert_eq!(outputs[0][0], 0.5);
        assert_eq!(outputs[1][0], 0.5);

        // 2. Change gain to 2.0
        prod.push(PatchEvent::SetParameter {
            param_id: "gain".to_string(),
            value: 2.0,
        })
        .unwrap();

        engine.process(&inputs, &mut outputs, 44100.0);

        assert_eq!(outputs[0][0], 2.0);
    }
}
