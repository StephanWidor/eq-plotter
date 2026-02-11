use crate::*;

#[derive(Debug, Clone, Copy)]
pub struct Coefficients<F: utils::Float> {
    pub attack: F,
    pub release: F,
}

impl<F: utils::Float> Coefficients<F> {
    pub fn new(attack_time: F, release_time: F, sample_rate: F) -> Self {
        Self {
            attack: Self::time_to_coefficient(attack_time, sample_rate),
            release: Self::time_to_coefficient(release_time, sample_rate),
        }
    }

    pub fn time_to_coefficient(time: F, sample_rate: F) -> F {
        if time <= F::zero() {
            F::one()
        } else {
            F::one() - (-F::one() / (time * sample_rate)).exp()
        }
    }
}

pub struct EnvelopeFollower<F: utils::Float> {
    coefficients: Coefficients<F>,
    out_state: F,
}

impl<F: utils::Float> EnvelopeFollower<F> {
    pub fn new(attack_time: F, release_time: F, sample_rate: F) -> Self {
        Self::from_coefficients(&Coefficients::new(attack_time, release_time, sample_rate))
    }

    pub fn from_coefficients(coefficients: &Coefficients<F>) -> Self {
        Self {
            coefficients: *coefficients,
            out_state: F::zero(),
        }
    }

    pub fn process(&mut self, sample: F) -> F {
        let coefficient = if sample > self.out_state {
            self.coefficients.attack
        } else {
            self.coefficients.release
        };

        self.out_state += coefficient * (sample - self.out_state);
        self.out_state
    }

    pub fn reset(&mut self, value: F) {
        self.out_state = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn converges_to_target() {
        let sample_rate = 44100_f32;
        let mut envelope = EnvelopeFollower::new(0.01_f32, 0.01_f32, sample_rate);

        let target = 0.7_f32;
        let mut out = 0_f32;
        for _ in 0..20000 {
            out = envelope.process(target);
        }

        assert_approx_eq!(out, target, 1e-3_f32);
    }

    #[test]
    fn reaches_time_constant() {
        let sample_rate = 48000.0;
        let attack_time = 0.05;
        let mut env = EnvelopeFollower::new(attack_time, 0.1, sample_rate);

        let num_samples = (attack_time * sample_rate) as usize;
        for _ in 0..num_samples {
            env.process(1.0);
        }

        let expected = 1.0 - (-1.0_f64).exp();

        assert_approx_eq!(env.out_state, expected, 1e-3);
    }

    #[test]
    fn attack_is_monotonic() {
        let mut envelope = EnvelopeFollower::new(0.01, 0.1, 16000.0);

        let mut previous_out = 0.0;
        for _ in 0..1000 {
            let out = envelope.process(0.5);
            assert!(out >= previous_out);
            previous_out = out;
        }
    }

    #[test]
    fn release_is_monotonic() {
        let mut envelope = EnvelopeFollower::new(0.01, 0.1, 48000.0);
        envelope.reset(1.0);

        let mut previous_out = 1.0;
        for _ in 0..1000 {
            let out = envelope.process(0.2);
            assert!(out <= previous_out);
            previous_out = out;
        }
    }
}
