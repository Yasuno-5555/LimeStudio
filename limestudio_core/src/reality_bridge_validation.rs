#[cfg(test)]
mod tests {
    use crate::engine::{VoiceEvent, VoiceManager};
    use crate::graph::GraphBuilder;
    use crate::PatchEvent;
    use rtrb::RingBuffer;
    use std::f32::consts::PI;

    /// Stage 1: DC Signal Pass (Integrity Check)
    /// 「信号が1ミリも狂わず通過するか。それが信頼の第一歩。」
    #[test]
    fn stage1_dc_signal_integrity() {
        let mut builder = GraphBuilder::new();
        let input = builder.input_node();
        let output = builder.output_node();
        builder.connect(input, output); // Straight wire

        let graph = builder.build();
        let (_, cons) = RingBuffer::<PatchEvent>::new(64);
        let mut engine = VoiceManager::from_graph(&graph, cons, 1, 44100.0, None);
        engine.handle_event(VoiceEvent::NoteOn {
            pitch: 60,
            velocity: 1.0,
        });

        let samples = 512;
        let in_data = vec![0.12345f32; samples];
        let inputs = vec![in_data.as_slice(), in_data.as_slice()];
        let mut out_l = vec![0.0f32; samples];
        let mut out_r = vec![0.0f32; samples];
        let mut outputs = vec![out_l.as_mut_slice(), out_r.as_mut_slice()];

        engine.process(&inputs, &mut outputs, 44100.0);

        for (i, (l, r)) in out_l.iter().zip(out_r.iter()).enumerate().take(samples) {
            assert_eq!(
                *l, 0.12345f32,
                "Left channel integrity failed at sample {}",
                i
            );
            assert_eq!(
                *r, 0.12345f32,
                "Right channel integrity failed at sample {}",
                i
            );
        }
    }

    /// Stage 2: Sine Sweep Pass (DSP Correctness)
    /// 「周波数特性に嘘がないか。数式が正しくRealityに変換されているか。」
    #[test]
    fn stage2_sine_sweep_linearity() {
        let mut builder = GraphBuilder::new();
        let input = builder.input_node();
        let output = builder.output_node();
        builder.connect(input, output);

        let graph = builder.build();
        let (_, cons) = RingBuffer::<PatchEvent>::new(64);
        let mut engine = VoiceManager::from_graph(&graph, cons, 1, 44100.0, None);
        engine.handle_event(VoiceEvent::NoteOn {
            pitch: 60,
            velocity: 1.0,
        });

        let samples = 44100; // 1 second
        let mut in_data = vec![0.0f32; samples];
        for (i, val) in in_data.iter_mut().enumerate().take(samples) {
            let freq = 20.0 + (i as f32 / samples as f32) * 20000.0;
            *val = (i as f32 * 2.0 * PI * freq / 44100.0).sin();
        }

        let mut out_l = vec![0.0f32; samples];
        let mut out_r = vec![0.0f32; samples];

        // Process in blocks to simulate real usage
        let block_size = 128;
        for i in (0..samples).step_by(block_size) {
            let end = (i + block_size).min(samples);
            let inputs = vec![&in_data[i..end], &in_data[i..end]];
            let mut outputs = vec![&mut out_l[i..end], &mut out_r[i..end]];
            engine.process(&inputs, &mut outputs, 44100.0);
        }

        for i in 0..samples {
            let diff = (out_l[i] - in_data[i]).abs();
            assert!(
                diff < 1e-5,
                "Linearity failed at sample {}: diff={}",
                i,
                diff
            );
        }
    }

    /// Stage 3: Automation Stress (Host Sync & RT Safety)
    /// 「ホストからのスパムに耐えられるか。スレッドが悲鳴を上げないか。」
    #[test]
    fn stage3_automation_stress() {
        let mut builder = GraphBuilder::new();
        let input = builder.input_node();
        let output = builder.output_node();
        let gain = builder.add_processor(
            "Gain",
            vec![(
                "gain",
                crate::graph::ParamSource::Parameter("gain".to_string()),
            )],
        );
        builder.connect(input, gain);
        builder.connect(gain, output);

        let graph = builder.build();
        let (mut prod, cons) = RingBuffer::<PatchEvent>::new(1024);
        let mut engine = VoiceManager::from_graph(&graph, cons, 1, 44100.0, None);
        engine.handle_event(VoiceEvent::NoteOn {
            pitch: 60,
            velocity: 1.0,
        });

        let iterations = 1000;
        let in_l = vec![1.0f32; 64];
        let in_r = vec![1.0f32; 64];
        let inputs = vec![in_l.as_slice(), in_r.as_slice()];
        let mut out_l = vec![0.0f32; 64];
        let mut out_r = vec![0.0f32; 64];
        let mut outputs = vec![out_l.as_mut_slice(), out_r.as_mut_slice()];

        for i in 0..iterations {
            let val = (i as f32 * 0.001).sin();
            // Rapid automation spam
            let _ = prod.push(PatchEvent::SetParameter {
                param_id: "gain".to_string(),
                value: val,
            });

            engine.process(&inputs, &mut outputs, 44100.0);

            // Should not crash and should update value (eventually)
            assert!(outputs[0][0].is_finite());
        }
    }
}
