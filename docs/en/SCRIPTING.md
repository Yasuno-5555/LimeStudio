# LimeStudio Scripting Guide: The Sandbox

In LimeStudio, scripting is used to **generate the signal graph**, not to process samples. This unique approach ensures that your custom logic is as fast as the built-in nodes.

## 1. The Core Concept

A `Script` node in LimeStudio is a **Compile-time Generator**.
When you write a script, the Rhai engine executes it to build a sequence of **Intermediate Representation (IR)** instructions. These instructions are then executed by the high-performance Rust engine on every audio sample.

---

## 2. API Reference

The script has access to a restricted set of functions for building audio logic.

### Inputs and Outputs
- `input(idx)`: Returns a `Signal` handle for the input port at `idx`.
- `output(idx, signal)`: Connects a `Signal` to the output port at `idx`.

### Arithmetic Operations
- `add(sig1, sig2)`: Sums two signals. Returns a new `Signal`.
- `mul(sig, factor)`: Multiplies a signal by a constant factor. Returns a new `Signal`.

---

## 3. Example: Custom Gain

This is the "Hello World" of LimeStudio scripting.

```javascript
// 1. Get the input signal from port 0
let x = input(0);

// 2. Multiply it by 0.5 (half volume)
let y = mul(x, 0.5);

// 3. Send it to output port 0
output(0, y);
```

### What happens under the hood?
The script generates the following IR:
1. `LoadBuffer(Input_0)`
2. `MulConst(0.5)`
3. `StoreBuffer(Temp_1)`
4. `CopyBuffer(Temp_1, Output_0)`

---

## 4. Why this is safe
- **No Infinite Loops**: The script only runs once during compilation. If it takes too long, the compiler times out.
- **No RT-Violations**: The script doesn't run in the audio thread. Only the generated IR runs there.
- **Deterministic**: The same script always generates the same IR.

## 5. Tips for advanced users
- **Signals are Buffers**: Each time you call `add` or `mul`, a temporary buffer is allocated by the compiler.
- **Static only**: Currently, scripting only supports static IR generation. Dynamic parameter modulation from scripts is planned for Level 3.
