pub mod saw {
    #[inline(always)]
    pub fn rising(phase: f32) -> f32 {
        phase * 2. - 1.
    }

    #[inline(always)]
    pub fn falling(phase: f32) -> f32 {
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
