use crate::eq;
use crate::utils;
use num_traits::Float;
use num_traits::cast::FromPrimitive;
use std::f64;

#[derive(Debug, Clone, Copy)]
pub struct Coefficients<F: Float + FromPrimitive> {
    pub a1: F,
    pub a2: F,
    pub b0: F,
    pub b1: F,
    pub b2: F,
}

/// Formulas for coefficients taken from here http://shepazu.github.io/Audio-EQ-Cookbook/audio-eq-cookbook.html
impl<F: Float + FromPrimitive> Coefficients<F> {
    pub fn from_volume_linear(volume_linear: F) -> Self {
        Self {
            a1: F::zero(),
            a2: F::zero(),
            b0: volume_linear,
            b1: F::zero(),
            b2: F::zero(),
        }
    }

    pub fn from_volume(volume: &eq::Volume<F>) -> Self {
        Self::from_volume_linear(utils::db_to_amplitude(volume.gain_db))
    }

    pub fn from_lowpass(lowpass: &eq::LowPass<F>, sample_rate: F) -> Self {
        let (alpha, cos_omega0) =
            Self::alpha_and_cos_omega0(lowpass.cutoff_frequency, lowpass.q, sample_rate);
        let one_minus_cos_omega0 = F::one() - cos_omega0;
        let half_one_minus_cos_omega0 = F::from(0.5).unwrap() * one_minus_cos_omega0;
        let a0 = F::one() + alpha;
        assert!(a0 != F::zero());
        let one_through_a0 = F::one() / a0;
        Self {
            a1: -one_through_a0 * F::from(2).unwrap() * cos_omega0,
            a2: one_through_a0 * (F::one() - alpha),
            b0: one_through_a0 * half_one_minus_cos_omega0,
            b1: one_through_a0 * one_minus_cos_omega0,
            b2: one_through_a0 * half_one_minus_cos_omega0,
        }
    }

    pub fn from_highpass(highpass: &eq::HighPass<F>, sample_rate: F) -> Self {
        let (alpha, cos_omega0) =
            Self::alpha_and_cos_omega0(highpass.cutoff_frequency, highpass.q, sample_rate);
        let one_plus_cos_omega0 = F::one() + cos_omega0;
        let half_one_plus_cos_omega0 = F::from(0.5).unwrap() * one_plus_cos_omega0;
        let a0 = F::one() + alpha;
        assert!(a0 != F::zero());
        let one_through_a0 = F::one() / a0;
        Self {
            a1: -one_through_a0 * F::from(2).unwrap() * cos_omega0,
            a2: one_through_a0 * (F::one() - alpha),
            b0: one_through_a0 * half_one_plus_cos_omega0,
            b1: -one_through_a0 * one_plus_cos_omega0,
            b2: one_through_a0 * half_one_plus_cos_omega0,
        }
    }

    pub fn from_bandpass(bandpass: &eq::BandPass<F>, sample_rate: F) -> Self {
        let (alpha, cos_omega0) =
            Self::alpha_and_cos_omega0(bandpass.frequency, bandpass.q, sample_rate);
        let a0 = F::one() + alpha;
        assert!(a0 != F::zero());
        let one_through_a0 = F::one() / a0;
        Self {
            a1: -one_through_a0 * F::from(2).unwrap() * cos_omega0,
            a2: one_through_a0 * (F::one() - alpha),
            b0: one_through_a0 * alpha,
            b1: F::zero(),
            b2: -one_through_a0 * alpha,
        }
    }

    pub fn from_allpass(allpass: &eq::AllPass<F>, sample_rate: F) -> Self {
        let (alpha, cos_omega0) =
            Self::alpha_and_cos_omega0(allpass.frequency, allpass.q, sample_rate);
        let two_times_cos_omega0 = F::from(2).unwrap() * cos_omega0;
        let one_plus_alpha = F::one() + alpha;
        let one_minus_alpha = F::one() - alpha;
        let a0 = one_plus_alpha;
        assert!(a0 != F::zero());
        let one_through_a0 = F::one() / a0;
        Self {
            b0: one_through_a0 * one_minus_alpha,
            b1: -one_through_a0 * two_times_cos_omega0,
            b2: F::one(),
            a1: -one_through_a0 * two_times_cos_omega0,
            a2: one_through_a0 * one_minus_alpha,
        }
    }

    pub fn from_notch(notch: &eq::Notch<F>, sample_rate: F) -> Self {
        let (alpha, cos_omega0) = Self::alpha_and_cos_omega0(notch.frequency, notch.q, sample_rate);
        let two_times_cos_omega0 = F::from(2).unwrap() * cos_omega0;
        let a0 = F::one() + alpha;
        assert!(a0 != F::zero());
        let one_through_a0 = F::one() / a0;

        Self {
            b0: one_through_a0,
            b1: -one_through_a0 * two_times_cos_omega0,
            b2: one_through_a0,
            a1: -one_through_a0 * two_times_cos_omega0,
            a2: one_through_a0 * (F::one() - alpha),
        }
    }

    pub fn from_peak_linear(frequency: F, gain_linear: F, q: F, sample_rate: F) -> Self {
        let a = F::sqrt(gain_linear);
        let (alpha, cos_omega0) = Self::alpha_and_cos_omega0(frequency, q, sample_rate);
        let two_times_cos_omega0 = F::from(2).unwrap() * cos_omega0;
        let alpha_times_a = alpha * a;
        let alpha_through_a = alpha / a;
        let a0 = F::one() + alpha_through_a;
        assert!(a0 != F::zero());
        let one_through_a0 = F::one() / a0;
        Self {
            b0: one_through_a0 * (F::one() + alpha_times_a),
            b1: -one_through_a0 * two_times_cos_omega0,
            b2: one_through_a0 * (F::one() - alpha_times_a),
            a1: -one_through_a0 * two_times_cos_omega0,
            a2: one_through_a0 * (F::one() - alpha_through_a),
        }
    }

    pub fn from_peak(peak: &eq::Peak<F>, sample_rate: F) -> Self {
        Self::from_peak_linear(
            peak.frequency,
            utils::db_to_amplitude(peak.gain_db),
            peak.q,
            sample_rate,
        )
    }

    pub fn from_lowshelf_linear(cutoff_frequency: F, gain_linear: F, q: F, sample_rate: F) -> Self {
        let a = F::sqrt(gain_linear);
        let sqrt_a = F::sqrt(a);
        let (alpha, cos_omega0) = Self::alpha_and_cos_omega0(cutoff_frequency, q, sample_rate);
        let a_plus_one = a + F::one();
        let a_minus_one = a - F::one();
        let two_times_sqrt_a_times_alpha = F::from(2).unwrap() * sqrt_a * alpha;
        let a0 = a_plus_one + a_minus_one * cos_omega0 + two_times_sqrt_a_times_alpha;
        assert!(a0 != F::zero());
        let one_through_a0 = F::one() / a0;

        Self {
            b0: one_through_a0
                * a
                * (a_plus_one - a_minus_one * cos_omega0 + two_times_sqrt_a_times_alpha),
            b1: one_through_a0 * F::from(2).unwrap() * a * (a_minus_one - a_plus_one * cos_omega0),
            b2: one_through_a0
                * a
                * (a_plus_one - a_minus_one * cos_omega0 - two_times_sqrt_a_times_alpha),
            a1: -one_through_a0 * F::from(2).unwrap() * (a_minus_one + a_plus_one * cos_omega0),
            a2: one_through_a0
                * (a_plus_one + a_minus_one * cos_omega0 - two_times_sqrt_a_times_alpha),
        }
    }

    pub fn from_lowshelf(lowshelf: &eq::LowShelf<F>, sample_rate: F) -> Self {
        Self::from_lowshelf_linear(
            lowshelf.cutoff_frequency,
            utils::db_to_amplitude(lowshelf.gain_db),
            lowshelf.q,
            sample_rate,
        )
    }

    pub fn from_highshelf_linear(
        cutoff_frequency: F,
        gain_linear: F,
        q: F,
        sample_rate: F,
    ) -> Self {
        let a = F::sqrt(gain_linear);
        let sqrt_a = F::sqrt(a);
        let (alpha, cos_omega0) = Self::alpha_and_cos_omega0(cutoff_frequency, q, sample_rate);
        let a_plus_one = a + F::one();
        let a_minus_one = a - F::one();
        let two_times_sqrt_a_times_alpha = F::from(2).unwrap() * sqrt_a * alpha;
        let a0 = a_plus_one - a_minus_one * cos_omega0 + two_times_sqrt_a_times_alpha;
        assert!(a0 != F::one());
        let one_through_a0 = F::one() / a0;

        Self {
            b0: one_through_a0
                * a
                * (a_plus_one + a_minus_one * cos_omega0 + two_times_sqrt_a_times_alpha),
            b1: -one_through_a0 * F::from(2).unwrap() * a * (a_minus_one + a_plus_one * cos_omega0),
            b2: one_through_a0
                * a
                * (a_plus_one + a_minus_one * cos_omega0 - two_times_sqrt_a_times_alpha),
            a1: one_through_a0 * F::from(2).unwrap() * (a_minus_one - a_plus_one * cos_omega0),
            a2: one_through_a0
                * (a_plus_one - a_minus_one * cos_omega0 - two_times_sqrt_a_times_alpha),
        }
    }

    pub fn from_highshelf(highshelf: &eq::HighShelf<F>, sample_rate: F) -> Self {
        Self::from_highshelf_linear(
            highshelf.cutoff_frequency,
            utils::db_to_amplitude(highshelf.gain_db),
            highshelf.q,
            sample_rate,
        )
    }

    pub fn from_eq(eq: &eq::EQ<F>, sample_rate: F) -> Self {
        match eq {
            eq::EQ::Volume(volume) => Self::from_volume(&volume),
            eq::EQ::LowPass(lowpass) => Self::from_lowpass(&lowpass, sample_rate),
            eq::EQ::HighPass(highpass) => Self::from_highpass(&highpass, sample_rate),
            eq::EQ::BandPass(bandpass) => Self::from_bandpass(&bandpass, sample_rate),
            eq::EQ::AllPass(allpass) => Self::from_allpass(&allpass, sample_rate),
            eq::EQ::Notch(notch) => Self::from_notch(&notch, sample_rate),
            eq::EQ::Peak(peak) => Self::from_peak(&peak, sample_rate),
            eq::EQ::LowShelf(lowshelf) => Self::from_lowshelf(&lowshelf, sample_rate),
            eq::EQ::HighShelf(highshelf) => Self::from_highshelf(&highshelf, sample_rate),
        }
    }

    fn alpha_and_cos_omega0(frequency: F, q: F, sample_rate: F) -> (F, F) {
        let omega0 =
            F::from(2).unwrap() * F::from(f64::consts::PI).unwrap() * frequency / sample_rate;
        let alpha = F::from(0.5).unwrap() * F::sin(omega0) / q;
        return (alpha, F::cos(omega0));
    }
}
