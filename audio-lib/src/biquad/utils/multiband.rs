use super::*;
use crate::biquad::coefficients::Coefficients;

pub type ProcessingType = crate::eq::MultibandType;

pub fn process<'a, F: utils::Float + 'a, Iter: Iterator<Item = &'a mut Filter<F>>>(
    filters: Iter,
    processing_type: ProcessingType,
    sample: F,
) -> F {
    match processing_type {
        ProcessingType::Sequential => sequential::process(filters, sample),
        ProcessingType::Parallel => parallel::process(filters, sample),
    }
}

pub fn make_frequency_response<
    F: utils::Float + 'static,
    C: Iterator<Item = Coefficients<F>> + 'static,
>(
    coefficients: C,
    processing_type: ProcessingType,
    sample_rate: F,
) -> Box<dyn Fn(F) -> Complex<F>> {
    match processing_type {
        ProcessingType::Sequential => Box::new(sequential::make_frequency_response(
            coefficients,
            sample_rate,
        )),
        ProcessingType::Parallel => {
            Box::new(parallel::make_frequency_response(coefficients, sample_rate))
        }
    }
}

pub fn impulse_response<F: utils::Float, C: Iterator<Item = Coefficients<F>>>(
    coefficients: C,
    processing_type: ProcessingType,
    eps: F,
    hold_length: usize,
    max_length: usize,
) -> Vec<F> {
    match processing_type {
        ProcessingType::Sequential => {
            sequential::impulse_response(coefficients, eps, hold_length, max_length)
        }
        ProcessingType::Parallel => {
            parallel::impulse_response(coefficients, eps, hold_length, max_length)
        }
    }
}

pub mod sequential {
    use super::*;
    pub fn process<'a, F: utils::Float + 'a, Iter: Iterator<Item = &'a mut Filter<F>>>(
        filters: Iter,
        sample: F,
    ) -> F {
        let mut output = sample;
        for filter in filters {
            output = filter.process(output);
        }
        output
    }

    pub fn make_frequency_response<F: utils::Float, C: Iterator<Item = Coefficients<F>>>(
        coefficients: C,
        sample_rate: F,
    ) -> impl Fn(F) -> Complex<F> {
        let transfer_functions = coefficients
            .into_iter()
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

    pub fn impulse_response<F: utils::Float, C: Iterator<Item = Coefficients<F>>>(
        coefficients: C,
        eps: F,
        hold_length: usize,
        max_length: usize,
    ) -> Vec<F> {
        let mut filters = coefficients
            .into_iter()
            .map(|c| Filter::new(c))
            .collect::<Vec<_>>();
        let mut process = |s| process(&mut filters.iter_mut(), s);
        utils::make_impulse_response(&mut process, eps, hold_length, max_length)
    }
}

pub mod parallel {
    use super::*;
    pub fn process<'a, F: utils::Float + 'a, Iter: Iterator<Item = &'a mut Filter<F>>>(
        filters: Iter,
        sample: F,
    ) -> F {
        let mut output = F::ZERO;
        let mut count = 0;
        for filter in filters {
            output += filter.process(sample);
            count += 1;
        }
        if count > 1 {
            output /= F::from(count).unwrap()
        }
        output
    }

    pub fn make_frequency_response<F: utils::Float, C: Iterator<Item = Coefficients<F>>>(
        coefficients: C,
        sample_rate: F,
    ) -> impl Fn(F) -> Complex<F> {
        let transfer_functions = coefficients
            .into_iter()
            .map(|c| make_transfer_function(c.clone()))
            .collect::<Vec<_>>();
        move |frequency| {
            let z1 = Complex::from_polar(F::ONE, -utils::omega(frequency, sample_rate));
            let mut sum = Complex::from(F::ZERO);
            let mut count = 0;
            for transfer_function in transfer_functions.iter() {
                sum = sum + transfer_function(z1);
                count += 1;
            }
            if count > 1 {
                sum = sum / F::from(count).unwrap();
            }
            sum
        }
    }

    pub fn impulse_response<F: utils::Float, C: Iterator<Item = Coefficients<F>>>(
        coefficients: C,
        eps: F,
        hold_length: usize,
        max_length: usize,
    ) -> Vec<F> {
        let mut filters = coefficients
            .into_iter()
            .map(|c| Filter::new(c))
            .collect::<Vec<_>>();
        let mut process = |s| process(&mut filters.iter_mut(), s);
        utils::make_impulse_response(&mut process, eps, hold_length, max_length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;
    use num::complex::ComplexFloat;

    #[test]
    fn validate_frequency_response_sequential() {
        let sample_rate = 44100.0;
        let coefficients = [
            Coefficients::from_bandpass(1000.0, 0.01, sample_rate),
            Coefficients::from_lowshelf_db(-2.7, 432.1, 5.2, sample_rate),
            Coefficients::from_highpass(100.0, 2.4, sample_rate),
        ];

        let single_band_responses = coefficients
            .iter()
            .map(|c| crate::biquad::make_frequency_response(c.clone(), sample_rate))
            .collect::<Vec<_>>();
        let multiband_response = sequential::make_frequency_response(
            coefficients.iter().map(|c| c.clone()),
            sample_rate,
        );

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

    #[test]
    fn validate_frequency_response_parallel() {
        let sample_rate = 44100.0;
        let coefficients = [
            Coefficients::from_bandpass(1000.0, 0.01, sample_rate),
            Coefficients::from_lowshelf_db(-2.7, 432.1, 5.2, sample_rate),
            Coefficients::from_highpass(100.0, 2.4, sample_rate),
        ];
        let multiband_factor = 1.0 / coefficients.len() as f64;

        let single_band_responses = coefficients
            .iter()
            .map(|c| crate::biquad::make_frequency_response(c.clone(), sample_rate))
            .collect::<Vec<_>>();
        let multiband_response =
            parallel::make_frequency_response(coefficients.into_iter(), sample_rate);

        for i in 1..200 {
            let frequency = (i * 100) as f64;
            let mut r0 = num::Complex::from(0.0);
            for r in single_band_responses.iter() {
                r0 = r0 + multiband_factor * r(frequency);
            }
            let r1 = multiband_response(frequency);
            assert_approx_eq!(r0, r1);
        }
    }
}
