# LimeStudio Mental Models: From Interaction to Reality

LimeStudio is built on strict architectural principles that might feel counter-intuitive if you come from traditional UI or audio frameworks. This guide explains the two primary "mental shifts" required to master LimeStudio.

---

## 1. Perception vs. Reality: The State Projection Model

In traditional UI programming, a widget (like a knob) usually "owns" its value. When you turn the knob, it updates its local state and notifies other components.

In LimeStudio, **widgets are hallucinations.** They own nothing.

### 1.1 The "No Local Truth" Principle
We follow a strict **Execution Boundary Controller** pattern. In the entire application, there is only one source of truth: the **Kernel**.

- **Perception (UI)**: A reflection of what we *think* the state is.
- **Reality (Kernel)**: The actual, forensic state running on the audio thread.

### 1.2 The Communication Loop
Instead of direct mutation, everything follows a mandatory 5-step loop:

1.  **Intent**: You drag a knob. The UI issues an `Intent::TweakParam { id: "gain", value: 0.5 }`.
2.  **Transaction**: The core (Transaction Layer) receives and validates the Intent (e.g., "Is 0.5 within range?").
3.  **Update Reality**: The Kernel updates the actual DSP parameter. This is the **only** place where the value truly exists.
4.  **Snapshot (Truth)**: The Kernel sends a snapshot of its state back to the UI.
5.  **Projection**: The UI receives the snapshot and updates the knob's position to 0.5.

---

## 2. Squeezer: The Bridge Between Perception and Reality

The **Squeezer IDE** is the environment designed to manage this "Perception vs. Reality" cycle intuitively.

Squeezer is not just an editor; it serves as:
- **Visual Causality**: Shows how an `Intent` propagates through the DSP graph and translates into the final output.
- **Live Patching**: Allows you to hot-reload the Reality (DSP) and Perception (UI) without stopping the audio.
- **Truth Monitoring**: Projects feedback from the audio engine back onto the UI so you can be confident in what your code is actually doing.

---

## 3. The Color of Truth (Oklab Enforcement)

LimeStudio forbids linear sRGB interpolation for dynamic UI elements (meters, analyzers, etc.).

### 3.1 The Problem: "Gray Smudge"
Humans don't perceive light linearly. In sRGB, the mathematical midpoint between two colors often looks "muddy" or "darker" than the original colors.

### 3.2 The Solution: Oklab
Oklab is a **perceptual color space**. It ensures:
1.  **Hue Consistency**: A transition from blue to white doesn't accidentally pass through purple.
2.  **Perceptually Linear Brightness**: The 50% midpoint between "bright" and "dark" actually *feels* like 50% brightness to the human eye.

In LimeStudio, **UI clarity is safety.** We choose visual determinism over "good enough" defaults.

---

## 4. Forensic Determinism

The LimeStudio audio engine prioritizes reproducibility over "mostly works." This is essential for troubleshooting in production and for academic-grade sound synthesis.

### 4.1 Deterministic Reality (PRNG)
Randomness in DSP should never depend on external non-deterministic sources (like system time). LimeStudio uses `DeterministicRng` (Xoshiro256++) for all internal PRNG, ensuring bit-identical output across platforms when given the same seed.

### 4.2 The Telemetry Bridge (The Pulse of Truth)
Knowing what is happening inside the audio thread (CPU load, clipping, NaNs) is critical. However, the UI thread must never peek into or lock the audio thread.

LimeStudio provides a **Forensic Telemetry Bridge**:
- **Lock-free One-way Communication**: Events stream from the Audio Thread to the UI Thread via an `rtrb` (RingBuffer).
- **Streaming Truth**: Unlike static Snapshots, this bridge allows real-time tracking of dynamic "Audit Events."
- **Audio First**: If the telemetry buffer fills up, the system drops old packets to ensure audio processing is never interrupted.

---

## Developer Summary
