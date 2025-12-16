#[cfg(test)]
use assert_approx_eq::assert_approx_eq;
use num::Complex;
use num_traits::Float;
use num_traits::cast::FromPrimitive;

#[allow(type_alias_bounds)]
pub type PolynomRoots<F: Float> = smallvec::SmallVec<[Complex<F>; 2]>;

pub fn amplitude_to_db<F: Float + FromPrimitive>(amplitude: F) -> F {
    if amplitude > F::zero() {
        F::from(20).unwrap() * F::log10(amplitude)
    } else {
        F::neg_infinity()
    }
}

pub fn db_to_amplitude<F: Float + FromPrimitive>(db: F) -> F {
    if db == F::neg_infinity() {
        F::zero()
    } else {
        F::from(10).unwrap().powf(db * F::from(0.05).unwrap())
    }
}

pub fn omega<F: Float + FromPrimitive>(frequency: F, sample_rate: F) -> F {
    F::from(2.0 * std::f64::consts::PI).unwrap() * (frequency / sample_rate)
}

/// complex roots of polynom c2*x^2 + c1*x + c0
pub fn polynom_roots<F: Float + FromPrimitive>(c2: F, c1: F, c0: F) -> PolynomRoots<F> {
    if c2 == F::zero() {
        if c1 == F::zero() {
            if c0 == F::zero() {
                // any x is a solution here, let's just return zero
                return PolynomRoots::from_elem(Complex::<F>::from(F::zero()), 1);
            }
            return PolynomRoots::new();
        }
        return PolynomRoots::from_elem(Complex::from(-c0 / c1), 1);
    }
    let p = c1 / c2;
    let q = c0 / c2;
    let root_arg = p * p / F::from(4).unwrap() - q;
    let p_half = Complex::from(p * F::from(0.5).unwrap());
    if root_arg == F::zero() {
        return PolynomRoots::from_elem(-p_half, 1);
    }

    let root_val = Complex::from(root_arg).sqrt();
    PolynomRoots::from_slice(&[-p_half - root_val, -p_half + root_val])
}

#[cfg(test)]
mod tests {
    use more_asserts::assert_le;
    use num::complex::ComplexFloat;

    use super::*;

    #[test]
    fn roundtrip_amplitude_db() {
        let n = 100;
        let step = 1.0 / (n as f64);
        for i in 0..(n + 1) {
            let amplitude = (i as f64) * step;
            let db = amplitude_to_db(amplitude);
            let amplitude_back = db_to_amplitude(db);
            assert_approx_eq!(amplitude, amplitude_back);
        }
    }

    #[test]
    fn check_explicit_polyom_roots() {
        let check_solutions = |solutions: PolynomRoots<f64>, expected: &Vec<Complex<f64>>| {
            assert_eq!(solutions.len(), expected.len());
            for solution in solutions.iter() {
                let find_index = expected
                    .into_iter()
                    .position(|x| (x - solution).abs() <= 1e-5);
                assert_ne!(find_index, None);
            }
        };

        check_solutions(polynom_roots(0.0, 0.0, 0.0), &vec![Complex::from(0.0)]);
        check_solutions(polynom_roots(0.0, 0.0, 0.1), &Vec::new());
        check_solutions(polynom_roots(0.0, 2.0, 6.0), &vec![Complex::from(-3.0)]);
        check_solutions(
            polynom_roots(1.6, 0.0, 0.4),
            &vec![Complex { re: 0.0, im: 0.5 }, Complex { re: 0.0, im: -0.5 }],
        );
        check_solutions(
            polynom_roots(1.0, 0.0, -4.0),
            &vec![Complex::from(-2.0), Complex::from(2.0)],
        );
    }

    #[test]
    fn validate_some_polyom_roots() {
        let check_solutions = |c2, c1, c0| {
            let solutions = polynom_roots(c2, c1, c0);
            for solution in solutions {
                let eval: Complex<f64> = solution * solution * c2 + solution * c1 + c0;
                let abs_eval = eval.abs();
                assert_le!(abs_eval, 1e-10);
            }
        };

        check_solutions(1.2, 2.3, 3.4);
        check_solutions(-17.2, 23.5, 0.004);
    }
}
