//! A set of utility functions to convert
//! to and from different data types.

/// Convert a number of ticks to ms, bpm, pitch...
/// A tick corresponds to a single audio clock tick,
/// i.e. the time of a single sample.
pub mod tick {
    /// Convert from a number of ticks to a number of milliseconds.
    #[inline(always)]
    pub fn to_millis(ticks: f32, rate: f32) -> f32 {
        assert!(rate > 0.);
        ticks * 1000. / rate
    }

    /// Convert from a number of milliseconds to a number of ticks.
    #[inline(always)]
    pub fn from_millis(ms: f32, rate: f32) -> f32 {
        ms * rate / 1000.
    }

    /// Convert from a number of ticks to a number of beats-per-minute.
    #[inline(always)]
    pub fn to_bpm(ticks: f32, rate: f32) -> f32 {
        assert!(ticks > 0.);
        rate * 60. / ticks
    }

    /// Convert from a number of beats-per-minute to a number of ticks.
    #[inline(always)]
    pub fn from_bpm(bpm: f32, rate: f32) -> f32 {
        assert!(bpm > 0.);
        rate * 60. / bpm
    }

    /// Convert from a number of ticks to a pitch in hertz.
    #[inline(always)]
    pub fn to_pitch(ticks: f32, rate: f32) -> f32 {
        assert!(ticks > 0.);
        rate / ticks
    }

    /// Convert from a pitch in hertz to a number of ticks.
    #[inline(always)]
    pub fn from_pitch(pitch: f32, rate: f32) -> f32 {
        assert!(pitch > 0.);
        rate / pitch
    }
}

///
pub mod pitch {
    #[cfg(not(feature = "std"))]
    use crate::F32Extension;

    /// From a frequency in hertz to a number of cycles per sample.
    #[inline(always)]
    pub fn to_cycles(freq: f32, rate: f32) -> f32 {
        freq / rate
    }

    ///
    #[inline(always)]
    pub fn from_cycles(cps: f32, rate: f32) -> f32 {
        cps * rate
    }

    ///
    #[inline(always)]
    pub fn to_midi(freq: f32) -> f32 {
        assert!(freq >= 0.);
        12. * (freq / 440.).log2() + 69.
    }

    ///
    #[inline(always)]
    pub fn from_midi(midi: f32) -> f32 {
        assert!(midi >= 0.);
        440. * 2_f32.powf((midi - 69.) / 12.)
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

///
pub mod db {
    #[cfg(not(feature = "std"))]
    use crate::F32Extension;

    ///
    #[inline(always)]
    pub fn to_gain(db: f32) -> f32 {
        if db <= -100. {
            return 0.;
        }
        10_f32.powf(db * 0.05)
    }

    ///
    #[inline(always)]
    pub fn from_gain(gain: f32) -> f32 {
        if gain <= 0. {
            return -100.;
        }
        20. * gain.log10()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use core::f32::consts::PI;

    #[test]
    fn tick_to_millis() {
        assert_eq!(tick::to_millis(48_000., 48_000.), 1000.);
        assert_eq!(tick::to_millis(24_000., 48_000.), 500.);
        assert_eq!(tick::to_millis(1_000., 100_000.), 10.);
        assert_eq!(tick::to_millis(48., 48_000.), 1.);
    }

    #[test]
    fn tick_from_millis() {
        assert_eq!(tick::from_millis(1., 48_000.), 48.);
        assert_eq!(tick::from_millis(500., 48_000.), 24_000.);
    }

    #[test]
    fn tick_to_bpm() {
        assert_eq!(tick::to_bpm(1., 1.), 60.);
        assert_eq!(tick::to_bpm(24_000., 48_000.), 120.);
        assert_eq!(tick::to_bpm(10_000., 48_000.), 288.);
    }

    #[test]
    fn tick_from_bpm() {
        assert_eq!(tick::from_bpm(1., 1.), 60.);
        assert_eq!(tick::from_bpm(60., 48_000.), 48_000.);
        assert_eq!(tick::from_bpm(120., 48_000.), 24_000.);
    }

    #[test]
    fn tick_to_pitch() {
        assert_eq!(tick::to_pitch(1., 48_000.), 48_000.);
        assert_eq!(tick::to_pitch(24_000., 48_000.), 2.);
        assert_eq!(tick::to_pitch(1_500., 48_000.), 32.);
    }

    #[test]
    fn tick_from_pitch() {
        assert_eq!(tick::from_pitch(1., 1.), 1.);
        assert_eq!(tick::from_pitch(1., 2.), 2.);
        assert_eq!(tick::from_pitch(32., 48_000.), 1_500.);
    }

    #[test]
    fn pitch_to_cycles_per_second() {
        assert_eq!(pitch::to_cycles(1., 1.), 1.);
        assert_eq!(pitch::to_cycles(1., 4.), 0.25);
        assert_eq!(pitch::to_cycles(48., 48_000.), 0.001);
    }

    #[test]
    fn pitch_from_cycles_per_second() {
        assert_eq!(pitch::from_cycles(1., 1.), 1.);
        assert_eq!(pitch::from_cycles(0.01, 48.), 0.48);
    }

    #[test]
    fn pitch_to_midi() {
        assert_eq!(pitch::to_midi(8372.018), 120.);
        assert_eq!(pitch::to_midi(440.), 69.);
        assert_eq!(pitch::to_midi(46.249302), 30.);
    }

    #[test]
    fn pitch_from_midi() {
        assert_eq!(pitch::from_midi(120.), 8372.018);
        assert_eq!(pitch::from_midi(69.), 440.);
        assert_eq!(pitch::from_midi(30.), 46.249302);
    }

    #[test]
    fn radians_to_degrees() {
        assert_eq!(rad::to_deg(0.), 0.);
        assert_eq!(rad::to_deg(PI / 4.), 45.);
        assert_eq!(rad::to_deg(PI / 2.), 90.);
        assert_eq!(rad::to_deg(PI), 180.);
        assert_eq!(rad::to_deg(3. * PI / 2.), 270.);
        assert_eq!(rad::to_deg(2. * PI), 360.);
    }

    #[test]
    fn radians_from_degrees() {
        assert_eq!(rad::from_deg(0.), 0.);
        assert_eq!(rad::from_deg(45.), PI / 4.);
        assert_eq!(rad::from_deg(90.), PI / 2.);
        assert_eq!(rad::from_deg(180.), PI);
        assert_eq!(rad::from_deg(270.), 3. * PI / 2.);
        assert_eq!(rad::from_deg(360.), 2. * PI);
    }

    fn assert_near(left: f32, right: f32) {
        const EPSILON: f32 = 0.03;
        assert!(left.abs() < (right.abs() + EPSILON));
        assert!(left.abs() > (right.abs() - EPSILON));
    }

    #[test]
    fn db_to_gain() {
        assert_near(db::to_gain(6.), 2.);
        assert_near(db::to_gain(3.), 1.41);
        assert_near(db::to_gain(0.), 1.);
        assert_near(db::to_gain(-3.), 0.71);
        assert_near(db::to_gain(-6.), 0.5);
    }

    #[test]
    fn db_from_gain() {
        assert_near(db::from_gain(2.), 6.);
        assert_near(db::from_gain(1.), 0.);
        assert_near(db::from_gain(0.5), -6.);
    }
}
