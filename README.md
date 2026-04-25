# LimeStudio: Visible Compiler for Audio Logic

> "As long as you are on this carriage, you don't need to worry about DAW crashes or thread safety. Just whip as much as you like and pursue the sound."

LimeStudio is a high-performance audio framework built in Rust, designed for musicians and developers who want to create professional-grade audio plugins without diving into the complexities of low-level DSP mathematics.

## 1. Mission Statement
**"A professional audio framework where musicians can create plugins without knowing math — VPL-first, completely open-source, and written in Rust."**

---

## 2. Core Philosophy

### VPL is a Visible Compiler
The Visual Programming Language (VPL) is not just a UI; it is an **AST Editor**. It transforms high-level logic into a linear, real-time safe **Intermediate Representation (IR)**.

### Safe Sandbox (Scripting)
The Sandbox (Rhai) is not about freedom; it's about **safe restriction**. Scripts are used to generate IR at compile-time, not to process samples directly in the audio thread.

### Guaranteed Real-Time Safety
The core execution engine (DspEngine) is a linear IR interpreter. It is guaranteed to be:
- **Allocation-free**: Zero heap allocations in the audio path.
- **Lock-free**: No mutexes or synchronization primitives in the hot path.
- **Deterministic**: Same graph and script always produce the same IR.

---

## 3. Architecture

```mermaid
graph TD
    VPL[egui VPL Editor] -->|JSON| Graph[AudioGraph AST]
    Script[Rhai Sandbox] -->|IR Builder| Graph
    Graph -->|Compile/Validate| IR[Linear IR Ops]
    IR -->|Interpreter| Engine[DspEngine (Rust)]
    Engine -->|Audio IO| DAW[Host DAW]
```

---

## 4. Components

- **`limestudio_core`**: The heart of the framework. Contains the IR definition, the `AudioGraph` AST, the compiler, and the `DspEngine` runtime.
- **`limestudio_vpl`**: The visual frontend. A "Visible Compiler" that lets you build and inspect audio logic in real-time.
- **`limestudio_plugin`**: Bridge to the outside world (NIH-plug), handling polyphony and host integration.
- **`limestudio_dsp`**: Optimized DSP primitives used by the standard library.

---

## 5. Getting Started

### Run the VPL Editor
Inspect and build your audio logic visually:
```bash
cargo run -p limestudio_vpl
```

### Scripting Example
Write a simple gain script in the Sandbox:
```javascript
let x = input(0); 
let y = mul(x, 0.5); 
output(0, y);
```
This script is compiled into optimized IR operations:
1. `LoadBuffer(0)`
2. `MulConst(0.5)`
3. `StoreBuffer(Temp)`
4. `CopyBuffer(Temp, Output)`

---

## 6. License
MIT - Clean Logic. Professional Sound.
