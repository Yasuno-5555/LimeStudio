//! StarterSynth: A simple polyphonic synthesizer template for LimeStudio.
//! This demonstrates the "Law of 8px" UI and deterministic DSP.

use limestudio_plugin::prelude::*;

#[limestudio_plugin]
pub struct StarterSynth {
    params: Arc<SynthParams>,
}

#[derive(Params)]
pub struct SynthParams {
    #[id = "cutoff"]
    pub cutoff: FloatParam,
    #[id = "resonance"]
    pub resonance: FloatParam,
    #[id = "gain"]
    pub gain: FloatParam,
}

impl LimeProcessor for StarterSynth {
    fn process(&mut self, buffer: &mut AudioBuffer, context: &ProcessContext) {
        // Deterministic Sine Wave generation (Placeholder)
        for channel in 0..buffer.channels() {
            let samples = buffer.get_channel_mut(channel);
            for (i, sample) in samples.iter_mut().enumerate() {
                let t = (context.sample_index + i as u64) as f32 / 44100.0;
                *sample = (t * 440.0 * 2.0 * std::f32::consts::PI).sin() * self.params.gain.value();
            }
        }
        
        // Audit: Record a forensic snapshot of this block
        context.audit_snapshot("ProcessBlock", &self.params);
    }
}

impl LimeUI for StarterSynth {
    fn build_ui(&self) -> Box<dyn Widget> {
        // This layout was exported from Squeezer Visual Designer
        view! {
            vbox![
                Label::new("STARTER SYNTH").secondary(),
                hbox![
                    Knob::new(self.params.cutoff).label("CUTOFF"),
                    Knob::new(self.params.resonance).label("RESONANCE"),
                    Slider::new(self.params.gain).label("GAIN"),
                ],
                Label::new("FORENSIC READY").secondary()
            ]
        }
    }
}
