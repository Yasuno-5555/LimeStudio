# LimeStudio Design Document: The Visible Compiler

## 1. Perception vs Reality: The Execution Boundary

LimeStudio separates the "Perception" (Human interaction) from the "Reality" (Machine execution) to ensure total real-time safety and deterministic behavior.

### 1.1 Perception Layer (Shell)
- **Crates**: `limestudio_vpl`, `limestudio_surface`.
- **Identity**: Uses `UiIndex` (u64) for GPU-optimized rendering.
- **Responsibility**: Manages interaction, layout, and visual state. It never modifies the Audio Graph directly; it only issues **Intents**.

### 1.2 Transaction Layer (Bridge)
- **Crates**: `limestudio_core`.
- **Identity**: Maps `UiIndex` <-> `ULID` via the `ViewCache`.
- **Responsibility**: Compiles UI Intents into Kernel Patches. Manages the optimistic UI state and reconciles it with Kernel Snapshots. It acts as the "Execution Boundary Controller".

### 1.3 Reality Layer (Kernel)
- **Crates**: `dirtydata-runtime`, `dirtydata-core`.
- **Identity**: Uses `ULID` for globally unique entity identification and content hashing (BLAKE3).
- **Responsibility**: The source of truth. Executes the audio graph using a **JIT Execution Plan**. Manages state history (Lineage) for total auditability.

---

## 2. JIT Compilation: The Execution Plan

To avoid the overhead of graph traversal in the audio callback, LimeStudio transforms the Graph Topology into a linear **Execution Plan**.

- **DspOp**: A set of primitive instructions (Math, State, Bounds) that can be executed in a tight loop.
- **PlanCompiler**: Topologically sorts the graph and assigns virtual registers to every edge.
- **No Smart Optimization**: The JIT strictly follows the "Form Change" principle—transforming the graph's structure into a sequence without attempting speculative or acoustic-aware optimizations that might compromise predictability.

---

## 3. Trust UI & Hostile Validation

### 3.1 Trust UI
UI elements are designed to expose the internal state of the kernel:
- **Modulation Rings**: Direct visualization of a parameter's modulation depth and current value.
- **Confidence Visualizer**: Shows the `ConfidenceScore` of the observed state (Verified vs Inferred).
- **Provenance Trace**: Every parameter value can be traced back to its origin in the patch history.

### 3.2 Hostile Validation
The kernel performs constant validation of the DSP state:
- **AssertRange**: Every node execution is bounded by safety checks.
- **Nan/Inf Isolation**: Detected errors are isolated to prevent feedback-loop-driven speaker damage.

---

## 5. Design Enforcement: The Law of Lime

Design in Lime Studio is not a suggestion; it is a compiled constraint.

### 5.1 Lime HIG v3.0 (Law Edition)
The **Human Interface Guidelines (HIG)** define the mandatory visual and behavioral rules for all plugins.
- **Matte & Solid**: Prohibits shadows, glows, and deceptive lighting.
- **Shape Over Color**: Ensures accessibility and clarity through geometric state changes.
- **8px Grid Law**: Enforces architecture-level spacing consistency.

### 5.2 LimeLint: The Trust Compiler
All designs are audited by `LimeLint` to ensure they do not lie to the user.
- **Visual Auditing**: Rejects non-compliant UI attributes.
- **Forensic Testimony**: Links presets to their exact source logic via `cargo lime verify`.
- **The Release Ritual**: Mandates a deterministic "Ship Ceremony" (`cargo lime release`) to ensure every delivered plugin is trustworthy.

---

## 4. The SDK & Exporter

The `dirtydata-sdk` defines the "Law" for all external plugins.
- **plugin! macro**: A Kernel DSL that allows developers to write pure DSP logic while abstracting away ABI and host details.
- **Transmuter**: Converts a visual graph into a standalone, buildable Rust project that uses the SDK contract.
