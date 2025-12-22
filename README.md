# Limestudio: Next-Generation Wavelet DSP Framework

Limestudio is a high-performance, modular audio signal processing engine built in Rust. It leverages Continuous Wavelet Transform (CWT) to provide a multi-resolution analysis and synthesis framework, specifically designed for next-generation audio plugins and research applications.

**Status**: Prototype Complete (v0.1.0)
**License**: MIT

---

## Philosophy

### 1. Structure over Hacks
Limestudio rejects "black magic" DSP. Every component is structurally defined:
*   **BlockGatherer**: Bridges the gap between arbitrary host buffer sizes and fixed DSP block requirements.
*   **WaveletProcessor**: A stateless, block-based pure function mapping `Input Block -> Output Block`.
*   **Zero-Allocation**: The "hot path" (audio processing loop) is guaranteed to be free of heap allocations.

### 2. Visibility
Audio processing should not be invisible. Limestudio integrates a high-performance, lock-free visualization bridge from day one.
*   **Listening**: Artifact-free reconstruction (Perfect Reconstruction).
*   **Seeing**: dB-accurate, GPU-accelerated Spectrogram.

### 3. Modularity
The core engine (`limestudio_dsp`) is completely decoupled from the plugin format (`limestudio_plugin`).
*   **Core**: Pure Rust traits and logic.
*   **Plugin**: `nih-plug` adapter for VST3/CLAP support.
*   **GUI**: `vizia` based editor for visualization and control.

---

## Architecture

```mermaid
graph TD
    DAW[DAW Host] -->|Audio IO| Wrapper[WaveletEngineWrapper]
    Wrapper -->|Input Stream| Gatherer[BlockGatherer]
    
    subgraph "Limestudio DSP Engine"
        Gatherer -->|Fixed Block| Analyzer[SpectralAnalyzer]
        Analyzer -->|Spectrum| Synth[ScaleSynthesizer (Parallel)]
        Synth -->|Processed Blocks| Gatherer
    end

    Wrapper -->|Lock-Free| Visualizer[Spectrogram Widget]
    Visualizer -->|Texture Update| GPU[GUI / GPU Canvas]
    
    Gatherer -->|Output Stream| DAW
```

## DSP Specifications

*   **Mother Wavelet**: Complex Morlet (`w0 = 6.0`) for optimal time-frequency trade-off.
*   **Scale Distribution**: Logarithmic (Musical) spacing.
*   **Parallelism**: `rayon` based data-parallel processing per scale.
*   **Latency**: Reported as `FrameSize - HopSize` (Deterministic).

## Workspaces

*   `limestudio_core`: Fundamental traits (`AudioProcessor`, `AudioBuffer`).
*   `limestudio_dsp`: The heart of the engine. CWT, FFT, OLA logic.
*   `limestudio_plugin`: Bridge to `nih-plug`.
*   `basic_thru`: The reference implementation. A 5-band Wavelet EQ/Thru plugin.

## Verification

To verify the engine:

```bash
# Build the reference plugin (optimized)
cargo build --release -p basic_thru
```

The resulting VST3/CLAP plugin provides:
1.  **Perfect Reconstruction**: Pass-through audio with zero coloration when gains are at 1.0.
2.  **Latency Reporting**: Correctly reports `FrameSize - HopSize` latency to the host.
3.  **Visualization**: Real-time Spectrogram.

---

*Clean Logic. Clear Sound.*
