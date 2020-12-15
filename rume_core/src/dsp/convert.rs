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
    pub fn to_bpm(ticks: f32, rate: f32) -> f32 {
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
        12. * (freq / 440.).log2() + 69.
    }

    ///
    #[inline(always)]
    pub fn from_midi(midi: f32) -> f32 {
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
    const LOG_TEN: f32 = 2.302585092994046;

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

    ///
    #[inline(always)]
    pub fn to_rms(db: f32) -> f32 {
        to_power_unscaled(db, 0.05, 485.)
    }

    ///
    #[inline(always)]
    pub fn from_rms(rms: f32) -> f32 {
        from_power_unscaled(rms, 20.)
    }

    ///
    #[inline(always)]
    pub fn to_pow(db: f32) -> f32 {
        to_power_unscaled(db, 0.1, 870.)
    }

    ///
    #[inline(always)]
    pub fn from_pow(power: f32) -> f32 {
        from_power_unscaled(power, 10.)
    }

    #[inline(always)]
    fn to_power_unscaled(db: f32, scale: f32, max: f32) -> f32 {
        if db <= 0. {
            return 0.;
        }
        if db > max {
            return max;
        }
        (LOG_TEN * scale).exp() * (db - 100.)
    }

    #[inline(always)]
    fn from_power_unscaled(power: f32, scale: f32) -> f32 {
        if power > 0. {
            let val = scale / LOG_TEN * power.log10();
            if val >= 0. {
                return val;
            }
        }
        0.
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use core::f32::consts::PI;

    #[test]
    fn pitch_to_cycles_per_second() {}

    #[test]
    fn pitch_from_cycles_per_second() {}

    #[test]
    fn pitch_to_midi() {
        println!("{}", pitch::to_midi(440.));
        println!("{}", pitch::from_midi(70.));
    }

    #[test]
    fn pitch_from_midi() {}

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
}
