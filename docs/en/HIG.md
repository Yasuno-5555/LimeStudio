# Lime HIG v3.0 — Not Advice, But Law

**Project:** Lime Studio  
**Version:** 3.0 (Enforcement Edition)  
**Target:** CLAP / VST3 / AU / Rust (wgpu / nih-plug)  
**Principles:** Matte, Precision, Determinism, Enforced Consistency

---

## 1. Core Philosophy

The Lime Studio UI is not a decoration. It is a **Precision Instrument**.
The interface exists to maintain attention, expose causality, and eliminate ambiguity.

Every widget must answer:
- What changed?
- Why did it change?
- Can the user trust it?

If the answer is unclear, the widget is **Invalid**.

---

## 2. Absolute Visual Rules

### 2.1 Matte & Solid
**Forbidden:**
- Glow, Bloom, Shadow
- Glossy gradients, Fake reflections
- Neumorphism, Decorative blur/particles

**Mandatory:**
- Flat surfaces, Solid fills
- Geometry-driven emphasis, Explicit borders

### 2.2 Shape Over Color
State must be communicated via geometry first. Color is secondary.
**Priority:** Shape > Motion > Contrast > Color

### 2.3 Color Space (Oklab)
All interpolation MUST happen in Oklab. sRGB interpolation is forbidden because muddy gradients collapse civilizations.

---

## 3. Timing & Motion Rules

### 3.1 Animation Standard
All UI transitions must use **Physical Motion (Critically Damped Spring)** by default.

**Standard Constants:**
- **Standard:** Stiffness 300.0, Damping 35.0 (General transitions)
- **Fast:** Stiffness 600.0, Damping 50.0 (Button feedback)
- **Slow:** Stiffness 150.0, Damping 25.0 (Background shifts)

Linear interpolation is forbidden unless intended to look "digitally unnatural."

---

## 4. Typography
- **Font:** JetBrains Mono (Only. No exceptions.)
- Numbers must be monospace and aligned.

---

## 5. Spacing (The 8px Law)
All spacing must be a multiple of **8px**.

| Element | Minimum |
| :--- | :--- |
| Hit Area | 32px |
| Panel Padding | 16px |
| Section Gap | 24px |

---

## 6. Enforcement
This document is an **executable law**.
The mandatory tool `cargo lime-lint` will verify:
- Spacing violations
- Animation violations (Linear vs Spring)
- Color space violations (Non-Oklab)
- Parameter authority violations

CI must fail on any violation. Professional audio requires **Certainty**, not vibes.
