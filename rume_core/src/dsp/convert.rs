//! A set of utility functions to convert
//! to and from different data types.

/// Convert a number of ticks to ms, bpm, pitch...
/// A tick corresponds to a single audio clock tick,
/// i.e. the time of a single sample.
pub mod tick {
    /// Convert from a number of ticks to a number of milliseconds.
    #[inline(always)]
    pub fn to_millis(ticks: f32, rate: f32) -> f32 {
        ticks * 1000. / rate
    }

    /// Convert from a number of milliseconds to a number of ticks.
    #[inline(always)]
    pub fn from_millis(ms: f32, rate: f32) -> f32 {
        ms * rate / 1000.
    }

    /// Convert from a number of ticks to a number of beats-per-minute.
    #[inline(always)]
    pub fn to_bpm(ms: f32, rate: f32) -> f32 {
        rate * 60. / ticks
    }

    /// Convert from a number of beats-per-minute to a number of ticks.
    #[inline(always)]
    pub fn from_bpm(bpm: f32, rate: f32) -> f32 {
        rate * 60. / bpm
    }

    /// Convert from a number of ticks to a pitch in hertz.
    #[inline(always)]
    pub fn to_pitch(ticks: f32, rate: f32) -> f32 {
        rate / ticks
    }

    /// Convert from a pitch in hertz to a number of ticks.
    #[inline(always)]
    pub fn from_pitch(pitch: f32, rate: f32) -> f32 {
        rate / pitch
    }
}

///
pub mod pitch {
    /// From a frequency in hertz to a number of cycles per sample.
    /// Takes a frequency and a sampling time.
    #[inline(always)]
    pub fn to_cycles(freq: f32, time: f32) -> f32 {
        freq * time
    }

    ///
    #[inline(always)]
    pub fn from_cycles(cps: f32, rate: f32) -> f32 {
        cps * rate
    }

    ///
    #[inline(always)]
    pub fn to_midi(freq: f32) -> f32 {
        12_f32 * (freq / 440.).log2() + 69.
    }

    ///
    #[inline(always)]
    pub fn from_midi(cps: f32, rate: f32) -> f32 {
        440. * ((midi - 69.) / 12.).powf(2.)
    }
}

///
pub mod rad {
    use core::f32::consts::PI;

    ///
    #[inline(always)]
    pub fn to_deg(rad: f32) -> f32 {
        rad * 180. / PI
    }

    ///
    #[inline(always)]
    pub fn from_deg(deg: f32) -> f32 {
        deg * PI / 180.
    }
}
