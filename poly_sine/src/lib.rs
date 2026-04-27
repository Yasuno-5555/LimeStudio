use limestudio_plugin::*;

plugin! {
    name: "Poly Sine",
    vendor: "LimeStudio",
    params: {
        attack: Float = 0.01 [0.001 .. 2.0],
        release: Float = 0.5 [0.001 .. 5.0],
    },
    dsp: |ctx| {
        // Frequency from MIDI Pitch + MPE Tuning
        let freq = m2f(ctx.Pitch, ctx.Tuning);
        let osc = SineVCO(freq);
        
        // Envelope triggered by Note Gate
        let env = ADSR(ctx.attack, 0.1.into(), 0.8.into(), ctx.release, ctx.Gate);
        
        // Final VCA with Pressure modulation
        let vca = Multiply(osc, env);
        
        // Note: In a real implementation, we'd have a way to multiply a NodeHandle by a ParamSource
        // For now, let's just use the vca directly.
        vca.into_chainable(ctx.input.builder) >> ctx.output
    },
    ui: |params| {
        vbox![
            Knob::new(params.attack).label("Attack"),
            Knob::new(params.release).label("Release"),
        ]
    }
}
