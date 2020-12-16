pub mod saw {
    #[inline(always)]
    pub fn rise(phase: f32) -> f32 {
        phase * 2. - 1.
    }

    #[inline(always)]
    pub fn fall(phase: f32) -> f32 {
        1. - phase * 2.
    }
}

#[inline(always)]
pub fn square(phase: f32) -> f32 {
    pwm(phase, 0.5)
}

#[inline(always)]
pub fn pwm(phase: f32, duty: f32) -> f32 {
    match phase < duty {
        true => 1.0,
        false => -1.0,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rising_saw() {
        assert_eq!(saw::rise(0.0), -1.);
        assert_eq!(saw::rise(0.25), -0.5);
        assert_eq!(saw::rise(0.5), 0.);
        assert_eq!(saw::rise(0.75), 0.5);
        assert_eq!(saw::rise(1.0), 1.);
    }

    #[test]
    fn falling_saw() {
        assert_eq!(saw::fall(0.0), 1.);
        assert_eq!(saw::fall(0.25), 0.5);
        assert_eq!(saw::fall(0.5), 0.);
        assert_eq!(saw::fall(0.75), -0.5);
        assert_eq!(saw::fall(1.0), -1.);
    }

    #[test]
    fn basic_square() {
        assert_eq!(square(0.0), 1.0);
        assert_eq!(square(0.1), 1.0);
        assert_eq!(square(0.2), 1.0);
        assert_eq!(square(0.3), 1.0);
        assert_eq!(square(0.4), 1.0);
        assert_eq!(square(0.499), 1.0);
        assert_eq!(square(0.5), -1.0);
        assert_eq!(square(0.6), -1.0);
        assert_eq!(square(0.7), -1.0);
        assert_eq!(square(0.8), -1.0);
        assert_eq!(square(0.9), -1.0);
        assert_eq!(square(1.0), -1.0);
    }

    #[test]
    fn pulse_width() {
        const N: usize = 100;
        for x in 0..N {
            let x = x as f32 / N as f32;
            assert_eq!(pwm(x, 0.5), square(x));

            for d in 0..N {
                let d = d as f32 / N as f32;
                if x < d {
                    assert_eq!(pwm(x, d), 1.0);
                } else {
                    assert_eq!(pwm(x, d), -1.0);
                }
            }
        }
    }
}
