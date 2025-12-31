use crate::biquad::coefficients::Coefficients;
use num_traits::Float;

#[derive(Debug)]
pub struct Filter<F: Float> {
    coefficients: Coefficients<F>,
    input_state: [F; 2],
    output_state: [F; 2],
}

impl<F: Float> Filter<F> {
    pub fn new(coefficients: &Coefficients<F>) -> Self {
        Self {
            coefficients: *coefficients,
            input_state: [F::zero(); 2],
            output_state: [F::zero(); 2],
        }
    }

    pub fn set_coefficients(&mut self, coefficients: Coefficients<F>, reset_state: bool) {
        self.coefficients = coefficients;
        if reset_state {
            self.input_state.fill(F::zero());
            self.output_state.fill(F::zero());
        }
    }

    pub fn process(&mut self, sample: F) -> F {
        let c = &self.coefficients;
        let in_state = &mut self.input_state;
        let out_state = &mut self.output_state;
        let processed = c.b0 * sample + c.b1 * in_state[0] + c.b2 * in_state[1]
            - c.a1 * out_state[0]
            - c.a2 * out_state[1];
        in_state[1] = in_state[0];
        in_state[0] = sample;
        out_state[1] = out_state[0];
        out_state[0] = processed;
        processed
    }
}
