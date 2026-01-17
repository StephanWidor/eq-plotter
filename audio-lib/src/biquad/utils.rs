use crate::biquad::coefficients::Coefficients;
use crate::biquad::filter::Filter;
use crate::utils;
use num::Complex;

pub fn process_sequential<F: utils::Float>(filters: &mut [Filter<F>], sample: F) -> F {
    let mut output = sample;
    for filter in filters.iter_mut() {
        output = filter.process(output);
    }
    output
}

pub fn make_transfer_function<F: utils::Float>(
    coefficients: &Coefficients<F>,
) -> impl Fn(num::Complex<F>) -> Complex<F> {
    |z: num::Complex<F>| {
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
    coefficients: &Coefficients<F>,
    sample_rate: F,
) -> impl Fn(F) -> Complex<F> {
    let transfer_function = make_transfer_function(&coefficients);
    move |frequency| {
        transfer_function(Complex::from_polar(
            F::ONE,
            -utils::omega(frequency, sample_rate),
        ))
    }
}

pub fn impulse_response<F: utils::Float>(
    process_function: &mut impl FnMut(F) -> F,
    eps: F,
    hold_length: usize,
    max_length: usize,
) -> Vec<F> {
    let mut response = Vec::new();
    response.push(process_function(F::ONE));
    let mut eps_count = 0;
    while eps_count <= hold_length && response.len() <= max_length {
        let filter_out = process_function(F::ZERO);
        if filter_out.abs() <= eps {
            eps_count += 1;
        } else {
            eps_count = 0;
        }
        response.push(filter_out);
    }
    response.resize(response.len() + 1 - eps_count, F::ZERO);

    response
}

pub fn impulse_response_for_coefficients<F: utils::Float>(
    coefficients: &Coefficients<F>,
    eps: F,
    hold_length: usize,
    max_length: usize,
) -> Vec<F> {
    let mut filter = Filter::new(coefficients);
    let mut process = |s| filter.process(s);
    impulse_response(&mut process, eps, hold_length, max_length)
}

pub mod multiband {
    use super::*;

    pub fn make_frequency_response<F: utils::Float>(
        coefficients: &[Coefficients<F>],
        sample_rate: F,
    ) -> impl Fn(F) -> Complex<F> {
        let transfer_functions = coefficients
            .iter()
            .map(|c| make_transfer_function(c))
            .collect::<Vec<_>>();
        move |frequency| {
            let z1 = Complex::from_polar(F::ONE, -utils::omega(frequency, sample_rate));
            let mut product = Complex::from(F::ONE);
            for transfer_function in transfer_functions.iter() {
                product = product * transfer_function(z1);
            }
            product
        }
    }

    pub fn impulse_response_for_coefficients<F: utils::Float>(
        coefficients: &[Coefficients<F>],
        eps: F,
        hold_length: usize,
        max_length: usize,
    ) -> Vec<F> {
        let mut filters = coefficients
            .iter()
            .map(|c| Filter::new(c))
            .collect::<Vec<_>>();
        let mut process = |s| process_sequential(&mut filters, s);
        super::impulse_response(&mut process, eps, hold_length, max_length)
    }
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

        let coefficients = Coefficients::from_volume_db(gain_db);
        let response = make_frequency_response(&coefficients, sample_rate)(100.0);

        let gain_db_back = amplitude_to_db(response.abs());
        assert_approx_eq!(gain_db, gain_db_back);
    }

    #[test]
    fn validate_lowpass() {
        let sample_rate = 48000.0;
        let coefficients = Coefficients::from_lowpass(1000.0, 0.7, sample_rate);

        let frequency_response = make_frequency_response(&coefficients, sample_rate);

        let mut gain_db_back = amplitude_to_db(frequency_response(50.0).abs());
        assert_approx_eq!(gain_db_back, 0.0, 5e-4);

        gain_db_back = amplitude_to_db(frequency_response(10000.0).abs());
        assert_le!(gain_db_back, -40.0);
    }

    #[test]
    fn validate_highpass() {
        let sample_rate = 48000.0;
        let coefficients = Coefficients::from_highpass(1000.0, 0.7, sample_rate);

        let calc_response = make_frequency_response(&coefficients, sample_rate);

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

        let calc_response = make_frequency_response(&coefficients, sample_rate);

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

        let calc_response = make_frequency_response(&coefficients, sample_rate);

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

        let calc_response = make_frequency_response(&coefficients, sample_rate);

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

        let calc_response = make_frequency_response(&coefficients, sample_rate);

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

        let calc_response = make_frequency_response(&coefficients, sample_rate);

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

        let calc_response = make_frequency_response(&coefficients, sample_rate);

        let mut gain_db_back = amplitude_to_db(calc_response(50.0).abs());
        assert_approx_eq!(gain_db_back, 0.0, 3e-4);

        gain_db_back = amplitude_to_db(calc_response(20000.0).abs());
        assert_approx_eq!(gain_db_back, gain_db, 1e-4);
    }

    #[test]
    fn validate_transfer_function_multiband() {
        let sample_rate = 44100.0;
        let coefficients = [
            Coefficients::from_bandpass(1000.0, 0.01, sample_rate),
            Coefficients::from_lowshelf_db(-2.7, 432.1, 5.2, sample_rate),
            Coefficients::from_highpass(100.0, 2.4, sample_rate),
        ];

        let single_band_responses = coefficients
            .iter()
            .map(|c| make_frequency_response(c, sample_rate))
            .collect::<Vec<_>>();
        let multiband_response = multiband::make_frequency_response(&coefficients, sample_rate);

        for i in 1..200 {
            let frequency = (i * 100) as f64;
            let mut r0 = num::Complex::from(1.0);
            for r in single_band_responses.iter() {
                r0 = r0 * r(frequency);
            }
            let r1 = multiband_response(frequency);
            assert_approx_eq!(r0, r1);
        }
    }
}
