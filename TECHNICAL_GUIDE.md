# LimeStudio Technical Guide: Extending the Framework

This guide is for developers who want to contribute to the core or extend the standard library.

## 1. Adding a new IR Opcode

If you need a new primitive (e.g., `Modulo`, `Log`, `Custom Filter`):

1.  **Define the Opcode**: Add a variant to the `IrOp` enum in [ir.rs](file:///Users/yasuno/projects/LimeStudio/limestudio_core/src/ir.rs).
2.  **Update Display**: Add the string representation in `impl Display for IrOp`.
3.  **Implement Logic**: Add the execution branch in `DspEngine::process` in [engine.rs](file:///Users/yasuno/projects/LimeStudio/limestudio_core/src/engine.rs). **CRITICAL: Must be RT-Safe.**
4.  **Register in Registry**: If it's a standalone node, add it to [registry.rs](file:///Users/yasuno/projects/LimeStudio/limestudio_core/src/registry.rs).

---

## 2. Adding a Standard Library Node

Stdlib nodes are high-level abstractions over IR.

1.  **Define Node Type**: Add a variant to `StdlibNode` in [stdlib.rs](file:///Users/yasuno/projects/LimeStudio/limestudio_core/src/stdlib.rs).
2.  **Define Ports**: Update `input_ports()` and `output_ports()`.
3.  **Implement Compilation**: Update `compile()` to generate the `Vec<IrOp>`.
4.  **Add to Registry**: Add a `NodeDefinition` in [registry.rs](file:///Users/yasuno/projects/LimeStudio/limestudio_core/src/registry.rs) so it appears in the VPL.

---

## 3. The Compilation Pipeline

The `compile_graph` function in [compile.rs](file:///Users/yasuno/projects/LimeStudio/limestudio_core/src/compile.rs) follows these steps:
1.  **Topological Sort**: Determines the execution order based on edges.
2.  **Buffer Allocation**: Assigns a unique `BufferId` to every output port.
3.  **Opcode Generation**: Calls `compile()` on each node.
4.  **Buffer Management**: Injects `LoadBuffer` and `StoreBuffer` ops to move data between nodes.

---

## 4. Testing Requirements

All core changes must pass the following tests:
- **RT-Safe Audit**: No `Box`, `Vec` (pushing), `HashMap`, or `Mutex` in any function called by `DspEngine::process`.
- **JSON Roundtrip**: Ensure the graph can be serialized and deserialized without losing data (`test_json_roundtrip_compilation_parity`).
- **Bit-Identical IR**: Compilation must be deterministic.
