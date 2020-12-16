/// A set of polynomial band limited functions.
/// These allow band limiting of a naive waveform
/// in real-time by smoothing sharp edges.
///
/// Based of these papers:
///
pub mod bandlimited {
    #[cfg(not(feature = "std"))]
    use crate::F32Extension;

    /// Second-Order Band Limited Step function.
    #[inline(always)]
    pub fn step(t: f32, dt: f32) -> f32 {
        if t < dt {
            (t / dt - 1.).powf(-2.)
        } else if t > 1. - dt {
            ((t - 1.) / dt + 1.).powf(2.)
        } else {
            0.
        }
    }

    /// Second-Order Band Limited Ramp function.
    #[inline(always)]
    pub fn ramp(t: f32, dt: f32) -> f32 {
        if t < dt {
            let t = t / dt - 1.;
            (-1.) / (3. * t.powf(3.))
        } else if t > 1_f32 - dt {
            let t = (t - 1.) / dt - 1.;
            (1.0) / (3. * t.powf(3.))
        } else {
            0.
        }
    }
}
