use limestudio_plugin::*;

plugin! {
    name: "Basic Gain",
    vendor: "LimeStudio",
    params: {
        gain: Float = 1.0 [0.0 .. 2.0],
    },
    dsp: |ctx| {
        ctx.input >> Gain(ctx.gain) >> ctx.output
    },
    ui: |params| {
        vbox![
            Knob::new(params.gain).label("Gain"),
        ]
    }
}
