# LimeStudio Technical Guide: Architecture & Implementation

This guide provides a deep dive into the technical implementation of LimeStudio's "Visible Compiler" and "Trust UI" systems.

---

## 1. The DspEngine & Linear IR

The core of LimeStudio is a **Linear IR Interpreter**. This approach ensures real-time safety and determinism.

### 1.1 Instruction Set (IrOp)
Instructions are designed to be atomic and allocation-free:
- `LoadBuffer(id)` / `StoreBuffer(id)`: Explicit buffer management.
- `MulConst(v)` / `AddConst(v)`: Optimized math primitives.
- `Delay { samples, state_id }`: State-aware time processing.
- `ReadInput` / `WriteOutput`: IO abstraction.

### 1.2 Execution Loop
The engine processes blocks of audio using a fixed-size `SampleStack` (typically 64 samples) to minimize register pressure and maximize cache hits.

---

## 2. Live Compiler & Visible Intelligence

The "Visible Compiler" is implemented via a multi-stage feedback loop:

1.  **Topological Sort**: `validate_graph` ensures the graph is a Directed Acyclic Graph (DAG).
2.  **Linearization**: `compile_graph` converts the DAG into a flat sequence of `IrOp`.
3.  **Mapping**: During compilation, we generate a `node_to_ops` map, linking every UI node to its resulting IR instructions.
4.  **Codegen**: `ir_to_readable_rust` converts the IR into high-level variable-based Rust code for the user to inspect.

---

## 3. Lime Surface: The UI Runtime

Lime Surface is a custom UI operating system built for audio.

### 3.1 Physics-Based Interaction
Linear interpolation is strictly forbidden. We use **Critically Damped Springs**:
```rust
fn spring_update(current, target, velocity, dt, stiffness, damping) -> f32 {
    let force = (target - current) * stiffness;
    *velocity = (*velocity + force * dt) * damping;
    current + *velocity * dt
}
```
This ensures that every knob and fader has "weight" and "resistance", providing tactile feedback even on a screen.

### 3.2 SDF Rendering
We use **Signed Distance Fields (SDF)** for all geometric primitives.
- **Anti-aliasing**: Calculated in the fragment shader for pixel-perfect results at any zoom level.
- **Modulation Rings**: Rendered as concentric SDF arcs with perceptual color interpolation (Oklab).

---

## 4. Hostile Validation (Tier S Safety)

Real-time safety is not an afterthought. The `validate_hostile` function performs static analysis on the compiled IR:
1.  **Denormal Detection**: Identifies feedback loops where subnormal numbers might accumulate.
2.  **NaN Propagation**: Checks for potential division by zero or log(negative) scenarios.
3.  **Latency Tracking**: Sums total sample delay across every path in the graph.
4.  **Stack Overflow**: Verifies the depth of the IR stack against the engine's hard limits.

---

## 5. Preset & Migration System

Plugins must be "compatible with the future self".
- **Schema Version**: `u32` representing the data format (e.g., `v4`).
- **Migration Hooks**: Versioned functions that transform `v(N)` JSON into `v(N+1)` state.
- **Graph Diff**: A structural comparison engine that identifies specific node/port/parameter changes between two snapshots.

---

## 6. CLI & CI/CD Integration

The CLI toolset (`limestudio_cli`) is designed to be integrated into professional workflows:
- **`validate --hostile`**: Fails CI if a patch is not real-time safe.
- **`render`**: Generates a deterministic "Golden Reference" for regression testing.
- **`bench`**: Measures processing headroom to ensure the plugin runs on target hardware.

---

## 7. Implementation Roadmap (Reference)

- **Tier S**: Core reliability (Presets, Hostile, 3-rate).
- **Tier A**: Visibility & Transparency (Always Show Rust, Provenance).
- **Tier B**: Operational Efficiency (ParamGraph, Inspector).
- **Tier Ω**: Advanced Tooling (Graph Diff, Semantic Undo).
