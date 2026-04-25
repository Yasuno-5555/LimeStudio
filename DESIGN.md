# LimeStudio Design Document: The Visible Compiler

## 1. Multi-Layer Execution Model

LimeStudio separates the "Thinking" (Compilation/UI) from the "Doing" (Audio Processing).

### Level 2: Graph / VPL (AST)
The highest level where users interact. The graph is a directed acyclic graph (DAG) of `GraphNode`s.
- **Components**: `AudioGraph`, `NodeRegistry`, `VPL App`.
- **Responsibility**: User interaction, serialization to JSON, and structural validation.

### Level 1: Intermediate Representation (IR)
The bridge between human logic and machine execution.
- **Opcodes**: `LoadBuffer`, `StoreBuffer`, `Add`, `Mul`, `Delay`, etc.
- **Linearized**: A flat list of instructions.
- **Responsibility**: Topological sorting of the graph into an execution order.

### Level 0: DspEngine (Runtime)
The high-performance, real-time safe interpreter.
- **Architecture**: Stack-based + Buffer-indexed machine.
- **Constraints**: No allocations, no locks, no dynamic dispatch.
- **Responsibility**: Sample-by-sample execution of IR.

---

## 2. Polyphony & Voice Allocation

LimeStudio handles polyphony at the `DspEngine` level.
- **VoiceAllocator**: Manages voice states (Note On/Off) and stealing (Oldest-first).
- **Per-Voice Engine**: Each voice runs an independent instance of the `DspEngine` with its own buffer space and `CompiledGraph`.
- **Merging**: Voices are summed in the final stage before output.

---

## 3. Patching Semantics

To ensure stability and performance, LimeStudio enforces strict patching rules:
1. **Explicit Mixing**: You cannot connect two outputs to one input. You must use a `Mix` node. This avoids "implicit sum" bugs common in other VPLs.
2. **Port Type Safety**: Only compatible ports (e.g., Audio-to-Audio) can be connected. This is checked at both UI and Validation time.
3. **No Feedback Loops in Graph**: Feedback must be implemented using explicit `Delay` primitives at the IR level to ensure deterministic sample-accurate behavior.

---

## 4. The Sandbox (Scripting)

User-defined logic is implemented through **Compile-time IR Generation**.
- **Engine**: Rhai.
- **Input**: Script source + Port metadata.
- **Output**: `Vec<IrOp>`.
- **Security**: The script only has access to a "Tiny IR Builder" API. It cannot touch raw memory, perform I/O, or process audio samples directly.

---

## 5. Hot Reloading

Because the runtime is a simple IR interpreter, LimeStudio supports near-instant hot-reloading:
1. UI modifies `AudioGraph`.
2. Graph is compiled to new `CompiledGraph` IR.
3. `DspEngine` swaps the IR list at a safe point (usually at the start of a block).
4. Audio continues without a glitch.
