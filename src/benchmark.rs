//! Bounded frame-time sampling for benchmark tools. CPU wall-clock timing, not GPU timing.
use serde::Serialize;
#[derive(Default)]
pub struct FrameSamples { samples: Vec<f64> }
#[derive(Debug, Serialize, PartialEq)]
pub struct FrameReport { pub frames: usize, pub mean_ms: f64, pub p95_ms: f64, pub fps: f64 }
impl FrameSamples {
    pub const LIMIT: usize = 36000;
    pub fn record(&mut self, ms: f64) -> bool {
        if !ms.is_finite() || ms <= 0.0 || ms > 60000.0 || self.samples.len() >= Self::LIMIT { return false; }
        self.samples.push(ms); true
    }
    pub fn len(&self) -> usize { self.samples.len() }
    pub fn is_empty(&self) -> bool { self.samples.is_empty() }
    pub fn report(&self) -> Option<FrameReport> {
        if self.samples.is_empty() { return None; }
        let mut sorted = self.samples.clone();
        sorted.sort_by(f64::total_cmp);
        let n = sorted.len();
        let mean = sorted.iter().sum::<f64>() / n as f64;
        Some(FrameReport { frames:n, mean_ms:mean, p95_ms:sorted[(n*95).div_ceil(100)-1], fps:1000.0/mean })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn empty_and_invalid() { let mut s=FrameSamples::default(); assert!(!s.record(f64::NAN)); assert!(!s.record(0.0)); assert!(s.report().is_none()); }
    #[test] fn known_samples() { let mut s=FrameSamples::default(); for _ in 0..20 { assert!(s.record(20.0)); } let r=s.report().unwrap(); assert_eq!(r.fps,50.0); assert_eq!(r.p95_ms,20.0); }
    #[test] fn bounded() { let mut s=FrameSamples::default(); for _ in 0..FrameSamples::LIMIT { assert!(s.record(1.0)); } assert!(!s.record(1.0)); }
}
