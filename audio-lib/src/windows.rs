use crate::utils;

#[derive(Debug, Clone, Copy)]
pub enum WindowType {
    VonHann,
    Hamming,
    None,
}

pub fn center_value<F: utils::Float>(window_type: WindowType) -> F {
    match window_type {
        WindowType::VonHann => F::ONE_HALF,
        WindowType::Hamming => F::from(25.0 / 46.0).unwrap(),
        WindowType::None => F::ONE,
    }
}

pub fn make_window<F: utils::Float>(length: usize, window_type: WindowType) -> Vec<F> {
    make_cosine_window(length, center_value::<F>(window_type))
}

pub fn make_cosine_window<F: utils::Float>(length: usize, a0: F) -> Vec<F> {
    assert!(length > 1);
    let one_minus_a0 = F::ONE - a0;
    let step = F::TWO_PI / F::from(length - 1).unwrap();
    (0..length)
        .map(|i| a0 - one_minus_a0 * (F::from(i).unwrap() * step).cos())
        .collect()
}
