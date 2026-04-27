# LimeStudio Technical Guide: Architecture & Implementation

This guide provides a deep dive into the technical implementation of LimeStudio's "Perception vs Reality" architecture and the DirtyData kernel.

> [!NOTE]
> If you are new to the LimeStudio philosophy, start with the [Mental Models Guide](file:///Users/yasuno/projects/LimeStudio/MENTAL_MODELS.md) for a high-level explanation of our architectural decisions.

---

## 1. The Execution Boundary Controller

The core of the integration is the **Transaction Layer**, which acts as a strict guard between the UI and the Audio Engine.

### 1.1 Intent vs Patch
- **Intent**: An expression of user desire (e.g., `Intent::TweakParam { ui_index, value }`). Intents are produced by the UI.
- **Patch**: A structural modification to the Kernel (e.g., `Operation::ModifyNode { id, delta }`). Patches are compiled from Intents by the Transaction Layer.

### 1.2 ID Isolation (ViewCache)
To prevent "Zombie IDs" and ensure UI stability, we use a two-layer ID system:
- **UI Space**: `UiIndex` (u64). Stable, reusable indices optimized for GPU instance buffers.
- **Kernel Space**: `StableId` (ULID). Globally unique identifiers used for graph persistence and history.
- **BiMap**: The `ViewCache` maintains the mapping between these two spaces.

---

## 2. JIT Execution Plans

The **DirtyData JIT** transforms the graph into a flat sequence of instructions.

### 2.1 The Execution Loop
The engine executes `DspOp` instructions in a linear loop.
- **Registers**: Pre-allocated memory for audio buffers and state.
- **Atomic Operations**: `Add`, `Mul`, `Sin`, `Tanh`, `Accumulate`.
- **Deterministic PRNG**: Randomness inside the engine is controlled via `DeterministicRng` (Xoshiro256++). This ensures bit-identical audio output from the same seed.
- **Node Integration**: Legacy or specialized nodes are called via `CallNode` instructions, maintaining a stable execution plan while allowing for complex DSP logic.

### 2.2 Wait-Free Communication
All communication between the UI (Perception) and Audio (Reality) threads uses `rtrb` wait-free SPSC queues. This ensures that the audio thread is never blocked by UI events or memory allocations.

- **Command Queue**: One-way queue from UI to Engine. Carries patch updates and parameter changes (Intents).
- **Telemetry Bridge**: One-way high-speed event stream from Engine to UI. Reports CPU load, clipping, NaNs, and PRNG state changes in real-time.

---

## 3. SDK & Plugin Contract (Kernel Law)

The `dirtydata-sdk` is the source of truth for the plugin ABI.

### 3.1 The plugin! Macro
```rust
plugin! {
    name: "my_filter",
    params: {
        cutoff: { min: 20.0, max: 20000.0, default: 1000.0, label: "Cutoff" }
    },
    process: |ctx, inputs, outputs, self_p| {
        // Safe, block-based DSP logic
    }
}
```
This macro generates the necessary ABI boilerplate, manifest JSON, and static instance management, allowing developers to write "Perception-free" DSP code.

---

## 5. Forensic Auditing (The Three Sacred Treasures)

LimeStudio provides a suite of forensic tools to ensure the integrity and trust of audio software.

### 5.1 LimeLint (The Static Judge)
`cargo lime lint` performs deep static analysis of the plugin's source code and UI definitions.
- **Visual Law**: Rejects non-compliant HIG v3.0 attributes.
- **Architectural Integrity**: Detects local state mutation that bypasses the parameter authority.
- **Realtime Safety**: Identifies non-realtime-safe calls (allocations, locks, I/O) in the DSP path.

### 5.2 Lime Doctor (Survival Diagnosis)
`cargo lime doctor` audits the plugin against known host-specific anomalies.
- **Host Profiles**: Simulates lifecycles for Logic Pro, FL Studio, and Ableton.
- **Anti-Pattern Detection**: Flags "JUCE-smell" such as state duplication and manual layout calculation.

### 5.3 Lime Verify & Testify (Forensic Proof)
- **`verify`**: Cryptographically proves that a `.lime` preset matches its reported hashes.
- **`testify`**: Generates a forensic report linking a preset to its exact Git commit and DSP source code.

---

## 6. The Release Ritual (The Ship Ceremony)

The `cargo lime release` command is the mandatory procedure for production delivery.
1.  **Strict Audit**: Executes `lint --strict` and `doctor`.
2.  **Preset Validation**: Verifies all included factory presets via `verify`.
3.  **Release Build**: Compiles the final binary with optimizations.
4.  **Sealing**: Generates a `SHIPMENT_MANIFEST.txt` containing the environment and integrity hashes.

---

## 4. Operational Safety (Hostile Validation)

Safety is integrated into every instruction:
- **AssertRange**: Injected by the compiler to detect out-of-bounds signals at runtime.
- **AuditReport**: The kernel generates diagnostic records for every violation, which are then reflected back to the UI's safety monitors.
- **Lineage Rollback**: Because every change is a patch, we can perform a "Snapshot Rollback" to restore the engine to a known-safe state if a violation occurs.
