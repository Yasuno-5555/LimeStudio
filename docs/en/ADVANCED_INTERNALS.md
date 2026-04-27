# LimeStudio Advanced Internals: The Reality Engine

This document is intended for advanced developers familiar with Rust's type system, memory models, and real-time systems. It dives into the heart of LimeStudio's implementation and architecture.

---

## 1. Perception vs Reality Reconciliation

The most critical abstraction in LimeStudio is the separation of "Optimistic Updates" and "Confirmation" via the **Transaction Layer**.

### 1.1 Intent Compilation
An `Intent` issued from the UI is assigned a `ULID`-based `trace_id` by the Transaction Layer and compiled into a `PatchSet`.
- **Optimistic Reflection**: Operations like `AddNode` or `SetParameter` are reflected immediately in the UI (Perception) without waiting for a kernel response.
- **Side Effect Mitigation**: Destructive operations like `RemoveNode` are held until the kernel confirms the transaction.

### 1.2 ID Isolation
- **UI Space**: Uses `u64`-based `UiIndex` for GPU instance buffer efficiency.
- **Kernel Space**: Uses `ULID`-based `StableId` for global consistency and history tracking.
- The `ViewCache` maintains this bi-directional mapping, ensuring UI identity is preserved even during engine restarts or graph restoration.

---

## 2. DirtyData Kernel: JIT & Execution Plan

`DirtyData`, the engine's heart, does not execute the graph structure directly; instead, it compiles it into a linear execution plan.

### 2.1 Anatomy of the Execution Loop
An `ExecutionPlan` consists of:
- **Registers**: Pre-allocated memory for audio buffers and inter-node communication.
- **DspOp**: Minimal DSP instructions.
- **Node Integration**: Heavy lifting or external libraries are called via `CallNode`, while register I/O remains managed by the compiler.

### 2.2 Wait-Free Communication (SPSC)
Communication between the UI thread and the audio thread uses `rtrb` (Wait-free SPSC queues).
- **UI -> Engine**: `PatchEvent` (parameter changes, structural changes).
- **Engine -> UI**: `Snapshot` (physical state, analysis data).
This completely eliminates locks and allocations within the audio thread.

---

## 3. Polyphonic Voice Management

The voice management in `limestudio_core/src/engine.rs` employs a sophisticated "Voice Stealing" algorithm.

### 3.1 Scoring System
The "importance" of each voice is calculated using the following formula (conceptual):
`Score = StateImportance + LevelBonus - AgePenalty`
- Idle voices have infinite scores (hardest to steal).
- Voices in release, low-volume voices, or older voices are prioritized for stealing.

### 3.2 Click-Free Stealing State
When stealing a voice, LimeStudio doesn't reset the waveform immediately.
1. **Transition to Stealing**: Initiates a 220-sample fade-out on the target voice.
2. **Gate Off**: Sets the `gate` parameter to 0.0.
3. **Completion**: Clears registers and starts the new note after the fade-out completes.

---

## 4. `plugin!` Macro Metaprogramming

The `limestudio_macro` is not just a boilerplate generator; it acts as a **Static Analysis Injector**.

- **Static Inspection**: Analyzes code within the `dsp` block to detect non-realtime-safe calls (`println!`, `Vec::push`, etc.).
- **Automatic Instrumentation**: Injects a `CodeView` widget into the UI, allowing users to "see" the exact DSP code being executed.
- **ABI Bridge**: Hides the complexity of `nih-plug` lifecycles and automates the mapping between `Params` structs and `PatchEvent`s.

---

## 5. SDF-Based UI Runtime

`limestudio_surface` is a `wgpu` implementation using **Signed Distance Fields (SDF)** rather than bitmaps or traditional vector drawing.

- **Resolution Independence**: Perfect edges at any scale with zero aliasing.
- **Low Latency**: Shapes are generated directly in the GPU's fragment shaders, resulting in extremely low CPU overhead.
- **Perceptual Consistency**: All color interpolation occurs in `Oklab` space, eliminating sRGB "muddiness."

---

## Challenge for Developers

The internals of LimeStudio are a battle between **"Immutability (Reality)"** and **"Fluidity (Perception)."** If you wish to add a new `DspOp` or optimize the SDF logic, you are effectively "rewriting reality."

Run the stress rig in `limestudio_core/src/torture.rs` to ensure your changes maintain "Trust" even under extreme conditions.

---

## 6. Bypassing the Law: Developing without LimeLint

While LimeLint ensures "Trust" and "Compliance," advanced developers may need to bypass these restrictions for research or experimental UI paradigms.

### 6.1 Disabling Specific Laws
You can modify the `lime-lint.toml` in your project root to silence specific checkers:
- **UI Flexibility**: Set `visual.forbidden_attributes = []` to allow shadows and glows.
- **Architectural Freedom**: Set `architectural.enforce_parameter_authority = false` to allow local state in widgets.

### 6.2 The "Outlaw" Mode
For complete freedom, you can bypass the `limestudio_plugin` high-level SDK entirely:
1.  **Direct Kernel Integration**: Use `dirtydata-runtime` directly in a standard `nih-plug` project.
2.  **Manual Execution Plan**: Build your own `ExecutionPlan` using the `PlanCompiler` and execute it in your `process` loop.
3.  **UI Freedom**: Use `limestudio_surface` as a raw `wgpu` renderer without the `Widget` IR restrictions.

> [!CAUTION]
> Bypassing LimeLint removes the "Confidence" guarantee. The `cargo lime release` command will flag these builds as **"Unverified/Outlaw"** in the Shipment Manifest, and they may be rejected by "Trust-First" distribution platforms.
