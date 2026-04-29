use num_complex::Complex64;
use std::f64::consts::PI;

/// 全てのWavelet基底が実装すべきトレイト
pub trait MotherWavelet: Send + Sync {
    /// 時間領域での値を計算 (初期化やデバッグ用)
    fn time_domain(&self, t: f64) -> Complex64;

    /// 周波数領域での値を計算 (FFT畳み込み用)
    /// omega: 正規化角周波数 (0 ~ 2π)
    /// scale: スケールパラメータ
    fn frequency_domain(&self, omega: f64, scale: f64) -> f64;
}

/// Complex Morlet Wavelet
/// 音楽的な解析に最も適している
pub struct Morlet {
    pub center_frequency: f64, // w0
    pub bandwidth: f64,        // 帯域幅
}

impl Default for Morlet {
    fn default() -> Self {
        Self {
            center_frequency: 6.0, // 一般的な初期値
            bandwidth: 1.0,
        }
    }
}

impl MotherWavelet for Morlet {
    fn time_domain(&self, t: f64) -> Complex64 {
        // Ψ(t) = π^(-1/4) * e^(i*w0*t) * e^(-t^2/2)
        // term1: 正規化係数
        let term1 = PI.powf(-0.25);
        // term2: 複素正弦波 e^(i*w0*t)
        let term2 = Complex64::cis(self.center_frequency * t);
        // term3: ガウス窓 e^(-t^2/2)
        let term3 = (-t * t / 2.0).exp();

        Complex64::new(term1 * term3, 0.0) * term2
    }

    fn frequency_domain(&self, omega: f64, scale: f64) -> f64 {
        // フーリエ変換後のMorlet (解析解)
        // H(w) = π^(-1/4) * e^(-(w-w0)^2 / 2)
        // w = omega * scale (スケーリングされた角周波数)

        // 注意: omegaは 0 ~ 2π (または 0 ~ π) の範囲で渡されることが多い
        // ここでは連続時間フーリエ変換の解析解を用いる
        let w = omega * scale;
        let diff = w - self.center_frequency;

        // 正規化係数 (エネルギー保存のためにはスケールに応じた補正が必要だが、
        // MotherWavelet自体の形状定義としてはこれでよい)
        let norm = PI.powf(-0.25);

        norm * (-0.5 * diff * diff).exp()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_morlet_creation() {
        let morlet = Morlet::default();
        assert_eq!(morlet.center_frequency, 6.0);
    }

    #[test]
    fn test_morlet_time_domain() {
        let morlet = Morlet::default();
        let val = morlet.time_domain(0.0);
        // t=0で最大値付近になるはず
        // term1 = pi^-0.25 ≈ 0.7511
        // term2 = cis(0) = 1+0i
        // term3 = exp(0) = 1
        let expected = PI.powf(-0.25);
        assert!((val.re - expected).abs() < 1e-6);
        assert_eq!(val.im, 0.0);
    }
}
