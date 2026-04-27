# Lime Studio: Primitive Policy & Doctrine of Overlap

Drawing commands (Primitives) in Lime Studio are not merely geometry; they must adhere to "Laws" that prove the "provenance" and "responsibility" of data.

## 1. Primitive Admission Policy

To add a new primitive, all five of the following conditions must be met:

1.  **Semantic Independence**: The primitive itself must represent a unique meaning (e.g., Value, Connection, Instant).
2.  **Reuse Across Domains**: It must represent general semantics, not be exclusive to a specific widget.
3.  **Interaction Contract**: It must have a contract with the interaction system (e.g., Hit test, Snap, Focus).
4.  **Temporal Contract**: It must define its "interpolation strategy" (i.e., when it is allowed to lie).
5.  **Provenance Visibility**: Its data source must be auditable, contributing to the "Trust UI."

## 2. DensityMap Truth Policy (The Truth of Raster Data)

High-density data such as spectrograms are projected according to the following principles:

- **Freshness > Completeness**: Prioritize current truth (latestness) over temporal continuity.
- **Batch Accumulation**: Submit all slices accumulated since the last render in a single batch.
- **Trace Lost (Confession of Missing Evidence)**: If a buffer overrun occurs, do not hide it. Honestly render a visual marker (e.g., a red line) to indicate missing evidence.
- **Resolution Scaling**: Under high load, reduce resolution (Bins) instead of dropping frame rate.

## 3. Doctrine of Overlap

Prohibit color mixing via alpha blending; instead, render geometric "relationships."

### 1. Law of Total Response
For elements that should logically be additive (like parametric EQ bands), do not overlay translucent regions. Instead, render the **mathematically summed result (Union)** as a solid shape. Individual components should be shown only via outlines (Path).

### 2. Semantic Occlusion
When independent objects overlap, the front object must solidly hide the background. However, a **"Negative Space" (SDF subtraction margin)** must be provided at the boundary to clearly indicate the "fact" of the foreground-background relationship.

### 3. Boolean Intersection
When a "logical product" (Intersection) must be shown (e.g., selection ranges), use a **fixed step shift in Lightness (L) in Oklab space** instead of color mixing, rendering it as a third solid region.

## 4. Interaction & Transform Laws

Rules to maintain interaction consistency against geometric complexity.

- **Inverse Transform Law**:
  While drawing layers (TransformNode) perform "Local -> Screen" transformations, the Interaction Kernel must always hold the inverse matrix and pull Screen coordinates back to each layer's Local coordinates for hit-testing.
- **Interaction Clip Sovereignty**:
  A `ClipMask` acts on both rendering and interaction, but its scope can be defined independently. Semantic explicitness is required to avoid inconsistencies like "visible but unclickable."
- **Silent Honesty**:
  System error warnings like `Trace Lost` must not break the user's focus. Limit them to subtle expressions like notches at the edge of a spectrogram, and disclose full evidence only in "Forensic Mode."

## 5. Causal Replay Policy

A hybrid recording policy for tracing past causality.

- **Layer 1: Responsibility Events (Sovereignty)**:
  Record "incidents" such as rank changes, threshold crossings, and connection changes with 1ms precision. This serves as the core of the replay (The Court Record).
- **Layer 2: Sparse Snapshots (Auxiliary)**:
  To reconstruct smooth modulations, retain snapshots of major parameter movements at a frequency of approximately 10Hz.
- **Layer 3: Forensic Freeze (Exception)**:
  Upon user request, freeze-dry a specific segment at full resolution (60Hz or higher).
- **Law of Read-Only Evidence**:
  Editing past evidence (parameter states) during replay is strictly prohibited as it leads to the "collapse of civilization." Replay is always read-only.
