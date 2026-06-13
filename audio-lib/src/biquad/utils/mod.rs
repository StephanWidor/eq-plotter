pub mod multiband;

use crate::biquad::{coefficients::Coefficients, filter::*};
use crate::utils;
use num::Complex;

pub fn make_transfer_function<F: utils::Float>(
    coefficients: Coefficients<F>,
) -> impl Fn(num::Complex<F>) -> Complex<F> {
    move |z: num::Complex<F>| {
        let z_squared = z * z;
        let numerator = Complex::from(coefficients.b0)
            + Complex::from(coefficients.b1) * z
            + Complex::from(coefficients.b2) * z_squared;
        let denominator = Complex::<F>::ONE
            + Complex::from(coefficients.a1) * z
            + Complex::from(coefficients.a2) * z_squared;
        numerator / denominator
    }
}

pub fn make_frequency_response<F: utils::Float>(
    coefficients: Coefficients<F>,
    sample_rate: F,
) -> impl Fn(F) -> Complex<F> {
    let transfer_function = make_transfer_function(coefficients);
    move |frequency| {
        transfer_function(Complex::from_polar(
            F::ONE,
            -utils::omega(frequency, sample_rate),
        ))
    }
}

pub fn make_impulse_response<F: utils::Float>(
    coefficients: Coefficients<F>,
    eps: F,
    hold_length: usize,
    max_length: usize,
) -> Vec<F> {
    let mut filter = Filter::new(coefficients);
    let mut process = move |s| filter.process(s);
    utils::make_impulse_response(&mut process, eps, hold_length, max_length)
}

pub fn zeros<F: utils::Float>(coefficients: &Coefficients<F>) -> utils::PolynomRoots<F> {
    utils::polynom_roots(coefficients.b0, coefficients.b1, coefficients.b2)
}

pub fn poles<F: utils::Float>(coefficients: &Coefficients<F>) -> utils::PolynomRoots<F> {
    utils::polynom_roots(F::ONE, coefficients.a1, coefficients.a2)
}

pub fn is_stable<F: utils::Float>(coefficients: &Coefficients<F>) -> bool {
    let poles = poles(coefficients);
    poles
        .into_iter()
        .find(|pole: &Complex<F>| pole.norm() >= F::ONE)
        == None
}

#[cfg(test)]
mod tests {
    use crate::utils::amplitude_to_db;
    use assert_approx_eq::assert_approx_eq;
    use more_asserts::assert_le;
    use num::complex::ComplexFloat;

    use super::*;

    #[test]
    fn example_for_zeros() {
        let coefficients = Coefficients {
            a1: 3.0,
            a2: -3.0,
            b0: 3.2,
            b1: 0.0,
            b2: 3.2,
        };
        let zeros = zeros(&coefficients);

        assert_eq!(zeros.len(), 2);
        for zero_expected in [Complex { re: 0.0, im: 1.0 }, Complex { re: 0.0, im: -1.0 }] {
            assert!(zeros.iter().find(|zero| **zero == zero_expected).is_some());
        }
    }

    #[test]
    fn example_for_poles() {
        let coefficients = Coefficients {
            a1: 0_f32,
            a2: -1_f32,
            b0: 13.2_f32,
            b1: 1_f32,
            b2: -2.2_f32,
        };
        let poles = poles(&coefficients);

        assert_eq!(poles.len(), 2);
        for pole_expected in [
            Complex {
                re: 1_f32,
                im: 0_f32,
            },
            Complex {
                re: -1_f32,
                im: 0_f32,
            },
        ] {
            assert!(poles.iter().find(|pole| **pole == pole_expected).is_some());
        }
    }

    #[test]
    fn test_stability() {
        let stable_coefficients = Coefficients {
            a1: 0.0,
            a2: 0.25,
            b0: 1.6,
            b1: 0.8,
            b2: -0.4,
        };
        assert!(is_stable(&stable_coefficients));

        let unstable_coefficients = Coefficients {
            a1: 0.0,
            a2: 4.0,
            b0: 0.3,
            b1: -0.2,
            b2: 0.1,
        };
        assert!(!is_stable(&unstable_coefficients));
    }

    #[test]
    fn validate_volume() {
        let sample_rate = 44100.0;
        let gain_db = 2.3;

        let coefficients = Coefficients::from_volume_db(gain_db);
        let response = make_frequency_response(coefficients, sample_rate)(100.0);

        let gain_db_back = amplitude_to_db(response.abs());
        assert_approx_eq!(gain_db, gain_db_back);
    }

    #[test]
    fn validate_lowpass() {
        let sample_rate = 48000.0;
        let coefficients = Coefficients::from_lowpass(1000.0, 0.7, sample_rate);

        let frequency_response = make_frequency_response(coefficients, sample_rate);

        let mut gain_db_back = amplitude_to_db(frequency_response(50.0).abs());
        assert_approx_eq!(gain_db_back, 0.0, 5e-4);

        gain_db_back = amplitude_to_db(frequency_response(10000.0).abs());
        assert_le!(gain_db_back, -40.0);
    }

    #[test]
    fn validate_highpass() {
        let sample_rate = 48000.0;
        let coefficients = Coefficients::from_highpass(1000.0, 0.7, sample_rate);

        let calc_response = make_frequency_response(coefficients, sample_rate);

        let mut gain_db_back = amplitude_to_db(calc_response(50.0).abs());
        assert_le!(gain_db_back, -40.0);

        gain_db_back = amplitude_to_db(calc_response(15000.0).abs());
        assert_approx_eq!(gain_db_back, 0.0, 5e-4);
    }

    #[test]
    fn validate_bandpass() {
        let sample_rate = 48000.0;
        let frequency = 5000.0;
        let coefficients = Coefficients::from_bandpass(frequency, 10.0, sample_rate);

        let calc_response = make_frequency_response(coefficients, sample_rate);

        let mut gain_db_back = utils::amplitude_to_db(calc_response(50.0).abs());
        assert_le!(gain_db_back, -40.0);

        gain_db_back = utils::amplitude_to_db(calc_response(frequency).abs());
        assert_approx_eq!(gain_db_back, 0.0);

        gain_db_back = utils::amplitude_to_db(calc_response(20000.0).abs());
        assert_le!(gain_db_back, -40.0);
    }

    #[test]
    fn validate_allpass() {
        let sample_rate = 48000.0;
        let frequency = 5000.0;
        let coefficients = Coefficients::from_allpass(frequency, 10.0, sample_rate);

        let calc_response = make_frequency_response(coefficients, sample_rate);

        let mut gain_db_back = amplitude_to_db(calc_response(50.0).abs());
        assert_approx_eq!(gain_db_back, 0.0);

        gain_db_back = amplitude_to_db(calc_response(frequency).abs());
        assert_approx_eq!(gain_db_back, 0.0);

        gain_db_back = amplitude_to_db(calc_response(20000.0).abs());
        assert_approx_eq!(gain_db_back, 0.0);
    }

    #[test]
    fn validate_notch() {
        let sample_rate = 48000.0;
        let frequency = 5000.0;
        let coefficients = Coefficients::from_notch(frequency, 10.0, sample_rate);

        let calc_response = make_frequency_response(coefficients, sample_rate);

        let mut gain_db_back = amplitude_to_db(calc_response(50.0).abs());
        assert_approx_eq!(gain_db_back, 0.0, 1e-5);

        gain_db_back = amplitude_to_db(calc_response(frequency).abs());
        assert_le!(gain_db_back, -100.0);

        gain_db_back = amplitude_to_db(calc_response(20000.0).abs());
        assert_approx_eq!(gain_db_back, 0.0, 1e-3);
    }

    #[test]
    fn validate_peak() {
        let sample_rate = 48000.0;
        let frequency = 5000.0;
        let gain_db = 3.4;
        let coefficients = Coefficients::from_peak_db(gain_db, frequency, 10.0, sample_rate);

        let calc_response = make_frequency_response(coefficients, sample_rate);

        let mut gain_db_back = amplitude_to_db(calc_response(50.0).abs());
        assert_approx_eq!(gain_db_back, 0.0, 1e-5);

        gain_db_back = amplitude_to_db(calc_response(frequency).abs());
        assert_approx_eq!(gain_db_back, gain_db);

        gain_db_back = amplitude_to_db(calc_response(20000.0).abs());
        assert_approx_eq!(gain_db_back, 0.0, 1e-3);
    }

    #[test]
    fn validate_lowshelf() {
        let sample_rate = 48000.0;
        let gain_db = 3.4;
        let coefficients = Coefficients::from_lowshelf_db(gain_db, 1000.0, 0.7, sample_rate);

        let calc_response = make_frequency_response(coefficients, sample_rate);

        let mut gain_db_back = amplitude_to_db(calc_response(50.0).abs());
        assert_approx_eq!(gain_db_back, gain_db, 5e-4);

        gain_db_back = amplitude_to_db(calc_response(20000.0).abs());
        assert_approx_eq!(gain_db_back, 0.0, 1e-4);
    }

    #[test]
    fn validate_highshelf() {
        let sample_rate = 48000.0;
        let gain_db = -2.4;
        let coefficients = Coefficients::from_highshelf_db(gain_db, 1000.0, 0.7, sample_rate);

        let calc_response = make_frequency_response(coefficients, sample_rate);

        let mut gain_db_back = amplitude_to_db(calc_response(50.0).abs());
        assert_approx_eq!(gain_db_back, 0.0, 3e-4);

        gain_db_back = amplitude_to_db(calc_response(20000.0).abs());
        assert_approx_eq!(gain_db_back, gain_db, 1e-4);
    }
}
