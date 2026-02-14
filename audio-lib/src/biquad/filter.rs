use crate::biquad::coefficients::Coefficients;
use crate::utils;

#[derive(Debug)]
pub struct State<F: utils::Float> {
    input_state: [F; 2],
    output_state: [F; 2],
}

impl<F: utils::Float> State<F> {
    pub const fn new() -> Self {
        Self {
            input_state: [F::ZERO; 2],
            output_state: [F::ZERO; 2],
        }
    }

    pub fn process(&mut self, coefficients: &Coefficients<F>, sample: F) -> F {
        let in_state = &mut self.input_state;
        let out_state = &mut self.output_state;
        let processed = coefficients.b0 * sample
            + coefficients.b1 * in_state[0]
            + coefficients.b2 * in_state[1]
            - coefficients.a1 * out_state[0]
            - coefficients.a2 * out_state[1];
        in_state[1] = in_state[0];
        in_state[0] = sample;
        out_state[1] = out_state[0];
        out_state[0] = processed;
        processed
    }

    pub fn reset(&mut self) {
        self.input_state.fill(F::ZERO);
        self.output_state.fill(F::ZERO);
    }
}

#[derive(Debug)]
pub struct Filter<F: utils::Float> {
    coefficients: Coefficients<F>,
    state: State<F>,
}

impl<F: utils::Float> Filter<F> {
    pub const fn new(coefficients: &Coefficients<F>) -> Self {
        Self {
            coefficients: *coefficients,
            state: State::new(),
        }
    }

    pub fn set_coefficients(&mut self, coefficients: Coefficients<F>, reset_state: bool) {
        self.coefficients = coefficients;
        if reset_state {
            self.reset_state();
        }
    }

    pub fn process(&mut self, sample: F) -> F {
        self.state.process(&self.coefficients, sample)
    }

    pub fn reset_state(&mut self) {
        self.state.reset();
    }
}
