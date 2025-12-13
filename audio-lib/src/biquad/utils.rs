use crate::biquad::coefficients::Coefficients;
use crate::biquad::filter::Filter;
use crate::utils;
use num::Complex;
use num_traits::cast::FromPrimitive;

pub fn make_frequency_response_function<F: num_traits::Float + FromPrimitive>(
    coefficients: &Coefficients<F>,
    sample_rate: F,
) -> impl Fn(F) -> Complex<F> {
    move |frequency| {
        let omega = utils::omega(frequency, sample_rate);
        let z1 = Complex::from_polar(F::one(), -omega);
        let z2 = Complex::from_polar(F::one(), F::from(-2).unwrap() * omega);
        let numerator = Complex::from(coefficients.b0)
            + Complex::from(coefficients.b1) * z1
            + Complex::from(coefficients.b2) * z2;
        let denominator = Complex::from(F::one())
            + Complex::from(coefficients.a1) * z1
            + Complex::from(coefficients.a2) * z2;
        numerator / denominator
    }
}

pub fn impulse_response<F: num_traits::Float + FromPrimitive>(
    coefficients: &Coefficients<F>,
    eps: F,
    hold_length: usize,
    max_length: usize,
) -> Vec<F> {
    let mut filter = Filter::new(coefficients);
    let mut response = Vec::new();
    response.push(filter.process(F::one()));
    let mut eps_count = 0;
    while eps_count <= hold_length && response.len() <= max_length {
        let filter_out = filter.process(F::zero());
        if filter_out.abs() <= eps {
            eps_count += 1;
        } else {
            eps_count = 0;
        }
        response.push(filter_out);
    }
    response.resize(response.len() + 1 - eps_count, F::zero());

    response
}

pub fn zeros<F: num_traits::Float + FromPrimitive>(
    coefficients: &Coefficients<F>,
) -> Vec<Complex<F>> {
    utils::polynom_roots(coefficients.b0, coefficients.b1, coefficients.b2)
}

pub fn poles<F: num_traits::Float + FromPrimitive>(
    coefficients: &Coefficients<F>,
) -> Vec<Complex<F>> {
    utils::polynom_roots(F::one(), coefficients.a1, coefficients.a2)
}

pub fn is_stable<F: num_traits::Float + FromPrimitive>(coefficients: &Coefficients<F>) -> bool {
    let poles = poles(coefficients);
    poles
        .into_iter()
        .find(|pole: &Complex<F>| pole.norm() >= F::one())
        == None
}

#[cfg(test)]
mod tests {
    use crate::{eq, utils::amplitude_to_db};
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
        assert_ne!(
            zeros
                .clone()
                .into_iter()
                .position(|zero| { zero == Complex { re: 0.0, im: 1.0 } }),
            None
        );
        assert_ne!(
            zeros
                .into_iter()
                .position(|zero| { zero == Complex { re: 0.0, im: -1.0 } }),
            None
        );
    }

    #[test]
    fn example_for_poles() {
        let coefficients = Coefficients {
            a1: 0.0_f32,
            a2: -1.0_f32,
            b0: 13.2_f32,
            b1: 1.0_f32,
            b2: -2.2_f32,
        };
        let poles = poles(&coefficients);
        assert_eq!(poles.len(), 2);
        assert_ne!(
            poles.clone().into_iter().position(|pole| {
                pole == Complex {
                    re: 1.0_f32,
                    im: 0.0_f32,
                }
            }),
            None
        );
        assert_ne!(
            poles.into_iter().position(|pole| {
                pole == Complex {
                    re: -1.0_f32,
                    im: 0.0_f32,
                }
            }),
            None
        );
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

        let coefficients = Coefficients::from_volume(&eq::Volume { gain_db: gain_db });
        let response = make_frequency_response_function(&coefficients, sample_rate)(100.0);

        let gain_db_back = amplitude_to_db(response.abs());
        assert_approx_eq!(gain_db, gain_db_back);
    }

    #[test]
    fn validate_lowpass() {
        let sample_rate = 48000.0;
        let coefficients = Coefficients::from_lowpass(
            &eq::LowPass {
                cutoff_frequency: 1000.0,
                q: 0.7,
            },
            sample_rate,
        );

        let frequency_response = make_frequency_response_function(&coefficients, sample_rate);

        let mut gain_db_back = amplitude_to_db(frequency_response(50.0).abs());
        assert_approx_eq!(gain_db_back, 0.0, 5e-4);

        gain_db_back = amplitude_to_db(frequency_response(10000.0).abs());
        assert_le!(gain_db_back, -40.0);
    }

    #[test]
    fn validate_highpass() {
        let sample_rate = 48000.0;
        let coefficients = Coefficients::from_highpass(
            &eq::HighPass {
                cutoff_frequency: 1000.0,
                q: 0.7,
            },
            sample_rate,
        );

        let calc_response = make_frequency_response_function(&coefficients, sample_rate);

        let mut gain_db_back = amplitude_to_db(calc_response(50.0).abs());
        assert_le!(gain_db_back, -40.0);

        gain_db_back = amplitude_to_db(calc_response(15000.0).abs());
        assert_approx_eq!(gain_db_back, 0.0, 5e-4);
    }

    #[test]
    fn validate_bandpass() {
        let sample_rate = 48000.0;
        let frequency = 5000.0;
        let coefficients = Coefficients::from_bandpass(
            &eq::BandPass {
                frequency: frequency,
                q: 10.0,
            },
            sample_rate,
        );

        let calc_response = make_frequency_response_function(&coefficients, sample_rate);

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
        let coefficients = Coefficients::from_allpass(
            &eq::AllPass {
                frequency: frequency,
                q: 10.0,
            },
            sample_rate,
        );

        let calc_response = make_frequency_response_function(&coefficients, sample_rate);

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
        let coefficients = Coefficients::from_notch(
            &eq::Notch {
                frequency: frequency,
                q: 10.0,
            },
            sample_rate,
        );

        let calc_response = make_frequency_response_function(&coefficients, sample_rate);

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
        let coefficients = Coefficients::from_peak(
            &eq::Peak {
                frequency: frequency,
                gain_db: gain_db,
                q: 10.0,
            },
            sample_rate,
        );

        let calc_response = make_frequency_response_function(&coefficients, sample_rate);

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
        let coefficients = Coefficients::from_lowshelf(
            &eq::LowShelf {
                cutoff_frequency: 1000.0,
                gain_db: 3.4,
                q: 0.7,
            },
            sample_rate,
        );

        let calc_response = make_frequency_response_function(&coefficients, sample_rate);

        let mut gain_db_back = amplitude_to_db(calc_response(50.0).abs());
        assert_approx_eq!(gain_db_back, gain_db, 5e-4);

        gain_db_back = amplitude_to_db(calc_response(20000.0).abs());
        assert_approx_eq!(gain_db_back, 0.0, 1e-4);
    }

    #[test]
    fn validate_highshelf() {
        let sample_rate = 48000.0;
        let gain_db = -2.4;
        let coefficients = Coefficients::from_highshelf(
            &eq::HighShelf {
                cutoff_frequency: 1000.0,
                gain_db: gain_db,
                q: 0.7,
            },
            sample_rate,
        );

        let calc_response = make_frequency_response_function(&coefficients, sample_rate);

        let mut gain_db_back = amplitude_to_db(calc_response(50.0).abs());
        assert_approx_eq!(gain_db_back, 0.0, 3e-4);

        gain_db_back = amplitude_to_db(calc_response(20000.0).abs());
        assert_approx_eq!(gain_db_back, gain_db, 1e-4);
    }
}
