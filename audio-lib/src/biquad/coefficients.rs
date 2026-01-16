use crate::eq;
use crate::utils;

#[derive(Debug, Clone, Copy)]
pub struct Coefficients<F: utils::Float> {
    pub a1: F,
    pub a2: F,
    pub b0: F,
    pub b1: F,
    pub b2: F,
}

/// Formulas for coefficients taken from here http://shepazu.github.io/Audio-EQ-Cookbook/audio-eq-cookbook.html
impl<F: utils::Float> Coefficients<F> {
    pub fn from_eq(eq: &eq::Eq<F>, sample_rate: F) -> Self {
        let gain_db = eq.gain.db();
        let frequency = eq.frequency.hz();
        match eq.eq_type {
            eq::EqType::Volume => Self::from_volume_db(gain_db),
            eq::EqType::LowPass => Self::from_lowpass(frequency, eq.q, sample_rate),
            eq::EqType::HighPass => Self::from_highpass(frequency, eq.q, sample_rate),
            eq::EqType::BandPass => Self::from_bandpass(frequency, eq.q, sample_rate),
            eq::EqType::AllPass => Self::from_allpass(frequency, eq.q, sample_rate),
            eq::EqType::Notch => Self::from_notch(frequency, eq.q, sample_rate),
            eq::EqType::Peak => Self::from_peak_db(gain_db, frequency, eq.q, sample_rate),
            eq::EqType::LowShelf => Self::from_lowshelf_db(gain_db, frequency, eq.q, sample_rate),
            eq::EqType::HighShelf => Self::from_highshelf_db(gain_db, frequency, eq.q, sample_rate),
            eq::EqType::Bypassed => Self::passthrough(),
        }
    }

    pub const fn muted() -> Self {
        Self::from_volume_linear(F::ZERO)
    }

    pub const fn passthrough() -> Self {
        Self::from_volume_linear(F::ONE)
    }

    pub const fn from_volume_linear(volume_linear: F) -> Self {
        Self {
            a1: F::ZERO,
            a2: F::ZERO,
            b0: volume_linear,
            b1: F::ZERO,
            b2: F::ZERO,
        }
    }

    pub fn from_volume_db(volume_db: F) -> Self {
        Self::from_volume_linear(utils::db_to_amplitude(volume_db))
    }

    pub fn from_lowpass(cutoff_frequency: F, q: F, sample_rate: F) -> Self {
        let (alpha, cos_omega0) = Self::alpha_and_cos_omega0(cutoff_frequency, q, sample_rate);
        let one_minus_cos_omega0 = F::ONE - cos_omega0;
        let half_one_minus_cos_omega0 = F::ONE_HALF * one_minus_cos_omega0;
        let a0 = F::ONE + alpha;
        assert!(a0 != F::ZERO);
        let one_through_a0 = F::ONE / a0;
        Self {
            a1: -one_through_a0 * F::TWO * cos_omega0,
            a2: one_through_a0 * (F::ONE - alpha),
            b0: one_through_a0 * half_one_minus_cos_omega0,
            b1: one_through_a0 * one_minus_cos_omega0,
            b2: one_through_a0 * half_one_minus_cos_omega0,
        }
    }

    pub fn from_highpass(cutoff_frequency: F, q: F, sample_rate: F) -> Self {
        let (alpha, cos_omega0) = Self::alpha_and_cos_omega0(cutoff_frequency, q, sample_rate);
        let one_plus_cos_omega0 = F::ONE + cos_omega0;
        let half_one_plus_cos_omega0 = F::ONE_HALF * one_plus_cos_omega0;
        let a0 = F::ONE + alpha;
        assert!(a0 != F::ZERO);
        let one_through_a0 = F::ONE / a0;
        Self {
            a1: -one_through_a0 * F::TWO * cos_omega0,
            a2: one_through_a0 * (F::ONE - alpha),
            b0: one_through_a0 * half_one_plus_cos_omega0,
            b1: -one_through_a0 * one_plus_cos_omega0,
            b2: one_through_a0 * half_one_plus_cos_omega0,
        }
    }

    pub fn from_bandpass(frequency: F, q: F, sample_rate: F) -> Self {
        let (alpha, cos_omega0) = Self::alpha_and_cos_omega0(frequency, q, sample_rate);
        let a0 = F::ONE + alpha;
        assert!(a0 != F::ZERO);
        let one_through_a0 = F::ONE / a0;
        Self {
            a1: -one_through_a0 * F::TWO * cos_omega0,
            a2: one_through_a0 * (F::ONE - alpha),
            b0: one_through_a0 * alpha,
            b1: F::ZERO,
            b2: -one_through_a0 * alpha,
        }
    }

    pub fn from_allpass(frequency: F, q: F, sample_rate: F) -> Self {
        let (alpha, cos_omega0) = Self::alpha_and_cos_omega0(frequency, q, sample_rate);
        let two_times_cos_omega0 = F::TWO * cos_omega0;
        let one_plus_alpha = F::ONE + alpha;
        let one_minus_alpha = F::ONE - alpha;
        let a0 = one_plus_alpha;
        assert!(a0 != F::ZERO);
        let one_through_a0 = F::ONE / a0;
        Self {
            b0: one_through_a0 * one_minus_alpha,
            b1: -one_through_a0 * two_times_cos_omega0,
            b2: F::ONE,
            a1: -one_through_a0 * two_times_cos_omega0,
            a2: one_through_a0 * one_minus_alpha,
        }
    }

    pub fn from_notch(frequency: F, q: F, sample_rate: F) -> Self {
        let (alpha, cos_omega0) = Self::alpha_and_cos_omega0(frequency, q, sample_rate);
        let two_times_cos_omega0 = F::TWO * cos_omega0;
        let a0 = F::ONE + alpha;
        assert!(a0 != F::ZERO);
        let one_through_a0 = F::ONE / a0;

        Self {
            b0: one_through_a0,
            b1: -one_through_a0 * two_times_cos_omega0,
            b2: one_through_a0,
            a1: -one_through_a0 * two_times_cos_omega0,
            a2: one_through_a0 * (F::ONE - alpha),
        }
    }

    pub fn from_peak_linear(gain_linear: F, frequency: F, q: F, sample_rate: F) -> Self {
        let a = F::sqrt(gain_linear);
        let (alpha, cos_omega0) = Self::alpha_and_cos_omega0(frequency, q, sample_rate);
        let two_times_cos_omega0 = F::TWO * cos_omega0;
        let alpha_times_a = alpha * a;
        let alpha_through_a = alpha / a;
        let a0 = F::ONE + alpha_through_a;
        assert!(a0 != F::ZERO);
        let one_through_a0 = F::ONE / a0;
        Self {
            b0: one_through_a0 * (F::ONE + alpha_times_a),
            b1: -one_through_a0 * two_times_cos_omega0,
            b2: one_through_a0 * (F::ONE - alpha_times_a),
            a1: -one_through_a0 * two_times_cos_omega0,
            a2: one_through_a0 * (F::ONE - alpha_through_a),
        }
    }

    pub fn from_peak_db(gain_db: F, frequency: F, q: F, sample_rate: F) -> Self {
        Self::from_peak_linear(utils::db_to_amplitude(gain_db), frequency, q, sample_rate)
    }

    pub fn from_lowshelf_linear(gain_linear: F, cutoff_frequency: F, q: F, sample_rate: F) -> Self {
        let a = F::sqrt(gain_linear);
        let sqrt_a = F::sqrt(a);
        let (alpha, cos_omega0) = Self::alpha_and_cos_omega0(cutoff_frequency, q, sample_rate);
        let a_plus_one = a + F::ONE;
        let a_minus_one = a - F::ONE;
        let two_times_sqrt_a_times_alpha = F::TWO * sqrt_a * alpha;
        let a0 = a_plus_one + a_minus_one * cos_omega0 + two_times_sqrt_a_times_alpha;
        assert!(a0 != F::ZERO);
        let one_through_a0 = F::ONE / a0;

        Self {
            b0: one_through_a0
                * a
                * (a_plus_one - a_minus_one * cos_omega0 + two_times_sqrt_a_times_alpha),
            b1: one_through_a0 * F::TWO * a * (a_minus_one - a_plus_one * cos_omega0),
            b2: one_through_a0
                * a
                * (a_plus_one - a_minus_one * cos_omega0 - two_times_sqrt_a_times_alpha),
            a1: -one_through_a0 * F::TWO * (a_minus_one + a_plus_one * cos_omega0),
            a2: one_through_a0
                * (a_plus_one + a_minus_one * cos_omega0 - two_times_sqrt_a_times_alpha),
        }
    }

    pub fn from_lowshelf_db(gain_db: F, cutoff_frequency: F, q: F, sample_rate: F) -> Self {
        Self::from_lowshelf_linear(
            utils::db_to_amplitude(gain_db),
            cutoff_frequency,
            q,
            sample_rate,
        )
    }

    pub fn from_highshelf_linear(
        gain_linear: F,
        cutoff_frequency: F,
        q: F,
        sample_rate: F,
    ) -> Self {
        let a = F::sqrt(gain_linear);
        let sqrt_a = F::sqrt(a);
        let (alpha, cos_omega0) = Self::alpha_and_cos_omega0(cutoff_frequency, q, sample_rate);
        let a_plus_one = a + F::ONE;
        let a_minus_one = a - F::ONE;
        let two_times_sqrt_a_times_alpha = F::TWO * sqrt_a * alpha;
        let a0 = a_plus_one - a_minus_one * cos_omega0 + two_times_sqrt_a_times_alpha;
        assert!(a0 != F::ONE);
        let one_through_a0 = F::ONE / a0;

        Self {
            b0: one_through_a0
                * a
                * (a_plus_one + a_minus_one * cos_omega0 + two_times_sqrt_a_times_alpha),
            b1: -one_through_a0 * F::TWO * a * (a_minus_one + a_plus_one * cos_omega0),
            b2: one_through_a0
                * a
                * (a_plus_one + a_minus_one * cos_omega0 - two_times_sqrt_a_times_alpha),
            a1: one_through_a0 * F::TWO * (a_minus_one - a_plus_one * cos_omega0),
            a2: one_through_a0
                * (a_plus_one - a_minus_one * cos_omega0 - two_times_sqrt_a_times_alpha),
        }
    }

    pub fn from_highshelf_db(gain_db: F, cutoff_frequency: F, q: F, sample_rate: F) -> Self {
        Self::from_highshelf_linear(
            utils::db_to_amplitude(gain_db),
            cutoff_frequency,
            q,
            sample_rate,
        )
    }

    fn alpha_and_cos_omega0(frequency: F, q: F, sample_rate: F) -> (F, F) {
        let omega0 = F::TWO_PI * frequency / sample_rate;
        let alpha = F::ONE_HALF * F::sin(omega0) / q;
        return (alpha, F::cos(omega0));
    }
}
