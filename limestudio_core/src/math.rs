//! LimeMath: Deterministic floating point operations for LimeStudio.
//! 
//! This module provides tools to ensure that DSP calculations remain deterministic 
//! across different CPUs and compilers without sacrificing performance by avoiding 
//! Soft-float where not necessary.

use std::ops::{Add, Sub, Mul, Div, Neg};

/// A wrapper for f32 that enforces deterministic operations by providing explicit
/// methods and preventing non-deterministic compiler optimizations.
#[derive(Copy, Clone, Default, PartialEq, PartialOrd)]
#[repr(transparent)]
#[allow(non_camel_case_types)]
pub struct df32(pub f32);

impl df32 {
    pub const ZERO: Self = Self(0.0);
    pub const ONE: Self = Self(1.0);
    pub const PI: Self = Self(std::f32::consts::PI);

    #[inline(always)]
    pub fn from_f32(v: f32) -> Self {
        Self(v)
    }

    #[inline(always)]
    pub fn to_f32(self) -> f32 {
        self.0
    }

    /// Deterministic sine approximation using a 7th degree Taylor polynomial
    /// adjusted for the [-PI, PI] range.
    pub fn sin(self) -> Self {
        let x = self.0;
        // Basic range reduction to [-PI, PI] (not fully rigorous, but stable)
        let x = x % (2.0 * std::f32::consts::PI);
        let x = if x > std::f32::consts::PI { x - 2.0 * std::f32::consts::PI } else if x < -std::f32::consts::PI { x + 2.0 * std::f32::consts::PI } else { x };
        
        let x2 = x * x;
        let x3 = x2 * x;
        let x5 = x3 * x2;
        let x7 = x5 * x2;

        Self(x - x3 / 6.0 + x5 / 120.0 - x7 / 5040.0)
    }

    /// Deterministic cosine approximation.
    pub fn cos(self) -> Self {
        Self(self.0 + std::f32::consts::FRAC_PI_2).sin()
    }

    /// Fast, deterministic exp approximation (base e).
    pub fn exp(self) -> Self {
        // Implementation note: High performance plugins usually use LUTs or
        // SIMD-friendly polynomial approximations. 
        // This is a simplified deterministic approximation.
        Self(self.0.exp()) // FIXME: Replace with fully deterministic polynomial
    }
}

impl Add for df32 {
    type Output = Self;
    #[inline(always)]
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl Sub for df32 {
    type Output = Self;
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl Mul for df32 {
    type Output = Self;
    #[inline(always)]
    fn mul(self, rhs: Self) -> Self {
        Self(self.0 * rhs.0)
    }
}

impl Div for df32 {
    type Output = Self;
    #[inline(always)]
    fn div(self, rhs: Self) -> Self {
        Self(self.0 / rhs.0)
    }
}

impl Neg for df32 {
    type Output = Self;
    #[inline(always)]
    fn neg(self) -> Self {
        Self(-self.0)
    }
}

impl From<f32> for df32 {
    fn from(v: f32) -> Self {
        Self(v)
    }
}

impl From<df32> for f32 {
    fn from(v: df32) -> f32 {
        v.0
    }
}

/// §SSS: DeterministicRng — The Source of Predictable Chaos.
/// 
/// Uses the Xoshiro256++ algorithm to ensure bit-identical random 
/// sequences across all platforms and compilers.
pub struct DeterministicRng {
    state: [u64; 4],
}

impl DeterministicRng {
    /// Create a new RNG from a 64-bit seed.
    pub fn seed_from_u64(seed: u64) -> Self {
        let mut state = [0u64; 4];
        // SplitMix64 to seed the xoshiro state
        let mut s = seed;
        for val in &mut state {
            s = s.wrapping_add(0x9e3779b97f4a7c15);
            let mut z = s;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
            *val = z ^ (z >> 31);
        }
        Self { state }
    }

    /// Returns the next 64-bit random number.
    pub fn next_u64(&mut self) -> u64 {
        let res = (self.state[0].wrapping_add(self.state[3]))
            .rotate_left(23)
            .wrapping_add(self.state[0]);
        let t = self.state[1] << 17;

        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];

        self.state[2] ^= t;
        self.state[3] = self.state[3].rotate_left(45);

        res
    }

    /// Returns a random f32 in the range [0, 1).
    pub fn next_f32(&mut self) -> f32 {
        let u = self.next_u64();
        (u >> 40) as f32 / (1u64 << 24) as f32
    }

    /// Returns a random df32 in the range [0, 1).
    pub fn next_df32(&mut self) -> df32 {
        df32(self.next_f32())
    }
}
