# Squeezer IDE Tutorial: The Visible Development Experience

This tutorial walks you through building a simple resonant filter using **Squeezer**, the integrated development environment for LimeStudio.

---

## 1. Launching Squeezer

First, build and run the IDE:

```bash
cargo run -p squeezer
```

You will see the Project Tree on the left and the VPL (Visual Programming Language) canvas in the center.

---

## 2. Building the DSP Graph (The Reality)

In the center canvas, right-click to add nodes and connect them:

1.  Add an **Input** node.
2.  Add a **Filter** node and connect the Input to it.
3.  Add an **Output** node and connect the Filter to it.
4.  Add two **Knob** nodes and connect them to the Filter's `Frequency` and `Resonance` ports.

You can hear the audio changing in real-time as you move the knobs. This is the **Reality Layer**.

---

## 3. UI Projection (The Perception)

Once the DSP is functional, define the **Perception Layer**—how the user interacts with your sound. Open `src/lib.rs` (or use the UI view in Squeezer) and describe your UI declaratively:

```rust
ui: |params| {
    view! {
        vbox![
            Label::new("SQUEEZER FILTER").secondary(),
            hbox![
                Knob::new(params.cutoff).label("CUTOFF"),
                Knob::new(params.resonance).label("RESONANCE"),
            ],
            Lens::new("output_viz", "spectrum"),
        ]
    }
}
```

Squeezer will instantly preview the UI based on this description.

---

## 4. Forensic Inspection

Squeezer allows you to visualize the internal logic of your plugin:

- **Causality Link**: Move a knob and see visual lines showing which DSP parameters are being influenced.
- **Confidence Visualizer**: See the compiler's confidence level for generated DSP instructions to catch potential errors early.

---

## 5. Exporting

When your prototype is ready, click the **BUILD** button or run the export command to generate production binaries (VST3/CLAP):

```bash
cargo lime export
```

The exported plugin retains the exact graph and UI you built in Squeezer, ensuring professional-grade reliability in any DAW.

---

## Next Steps

For a deeper dive into UI customization, check the [UI Tutorial](file:///Users/yasuno/projects/LimeStudio/docs/en/UI_TUTORIAL.md).
