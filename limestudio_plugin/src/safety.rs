//! Reality Safety Layer - 「Realityを狂気（Denormals / Starvation）から守る」

/// Real-time thread safety measures.
pub struct Safety;

impl Safety {
    /// Flush-To-Zero (FTZ) and Denormals-Are-Zero (DAZ) を有効化し、
    /// 微小な浮動小数点数によるCPUスパイクを防止します。
    #[inline(always)]
    pub fn enable_denormal_protection() {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            use std::arch::x86_64::*;
            // FTZ: bit 15, DAZ: bit 6
            _mm_setcsr(_mm_getcsr() | 0x8040);
        }

        // Note: aarch64 (Apple Silicon) does not have the same "Denormal penalty"
        // as x86, but many implementations still use FTZ via the FPCR register.
    }

    /// リアルタイムスレッドでのメモリアロケーション、ロック、I/Oが
    /// 発生していないかを監視するためのフック（将来の拡張用）。
    pub fn assert_rt_safe() {
        // TODO: Use a library like `assert_no_alloc` or custom TLS flags
        // to detect unsafe operations on the audio thread.
    }
}
