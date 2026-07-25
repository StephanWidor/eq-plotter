use super::*;
use delay::Delay;
use envelope_follower::EnvelopeFollower;

#[derive(Debug, Clone)]
pub struct Params<Knee: gain::Knee<F>, F: utils::Float> {
    pub knee: Knee::Params,
    pub attack_time: F,
    pub release_time: F,
    pub lookahead_time: F,
}

pub struct SimpleLimiter<Knee: gain::Knee<F>, const LOOKAHEAD_CAPACITY: usize, F: utils::Float> {
    envelope: EnvelopeFollower<F>,
    knee: Knee,
    delay: Delay<LOOKAHEAD_CAPACITY, F>,
    enabled: bool, // still adding lookahead latency if not enabled
}

impl<Knee: gain::Knee<F>, const LOOKAHEAD_CAPACITY: usize, F: utils::Float>
    SimpleLimiter<Knee, LOOKAHEAD_CAPACITY, F>
{
    pub fn new(params: Params<Knee, F>, sample_rate: F) -> Self {
        Self {
            envelope: EnvelopeFollower::from_attack_and_release_time(
                params.attack_time,
                params.release_time,
                sample_rate,
            ),
            knee: Knee::new(params.knee),
            delay: Delay::new(
                (params.lookahead_time * sample_rate)
                    .round()
                    .to_usize()
                    .unwrap(),
            ),
            enabled: true,
        }
    }

    pub fn new_passthrough() -> Self {
        Self {
            envelope: EnvelopeFollower::from_coefficients(&envelope_follower::Coefficients {
                attack: F::ONE,
                release: F::ONE,
            }),
            knee: Knee::new_inactive(),
            delay: Delay::new(0),
            enabled: true,
        }
    }

    pub fn set(&mut self, params: Params<Knee, F>, sample_rate: F) {
        self.envelope.set_coefficients(
            &envelope_follower::Coefficients::from_attack_and_release_time(
                params.attack_time,
                params.release_time,
                sample_rate,
            ),
        );
        self.knee.set(params.knee);
        self.delay.set_delay(
            (params.lookahead_time * sample_rate)
                .round()
                .to_usize()
                .unwrap(),
        );
        self.reset();
    }

    /// Still adding lookahead latency if not enabled
    pub fn set_enabled(&mut self, enabled: bool) {
        if enabled != self.enabled {
            self.envelope.reset(F::ZERO);
            self.enabled = enabled;
        }
    }

    pub fn process(&mut self, sample: F) -> F {
        let gain = if self.enabled {
            self.knee.gain(self.envelope.process(sample.abs()))
        } else {
            F::ONE
        };
        gain * self.delay.process(sample)
    }

    pub fn reset(&mut self) {
        self.envelope.reset(F::ZERO);
        self.delay.reset();
    }

    pub fn get_lookahead_samples(&self) -> usize {
        self.delay.get_delay_in_samples()
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_plot() {
//         let threshold_linear = 0.9_f32; //utils::db_to_amplitude(-3_f32);
//         let limiter_params = Params::<gain::SoftKnee<f32>, f32> {
//             knee: gain::SoftKneeParams::<f32> {
//                 threshold_linear: threshold_linear,
//                 compression_factor: 0.0,
//                 knee_ratio: 1.1,
//             },
//             attack_time: 0.0,
//             release_time: 0.02,
//             lookahead_time: 0.002,
//         };
//         let sample_rate = 48000_f32;
//         let mut limiter =
//             SimpleLimiter::<gain::SoftKnee<f32>, 5000, f32>::new(limiter_params, sample_rate);

//         let num_samples = sample_rate as usize;
//         let frequency = 260_f32;
//         let gain_linear = 2_f32;
//         let sine_sample = |i: usize| {
//             gain_linear * (std::f32::consts::TAU * frequency / sample_rate * i as f32).sin()
//         };

//         use plotters::prelude::*;
//         let root = BitMapBackend::new("/tmp/limiter.png", (1024, 1024)).into_drawing_area();
//         root.fill(&WHITE).unwrap();
//         let mut chart = ChartBuilder::on(&root)
//             .x_label_area_size(20)
//             .y_label_area_size(20)
//             .build_cartesian_2d(0f32..num_samples as f32, -2f32..2f32)
//             .unwrap();

//         chart.configure_mesh().draw().unwrap();

//         chart
//             .draw_series(LineSeries::new(
//                 (0..=num_samples).map(|i| {
//                     let s = sine_sample(i);
//                     (i as f32, s)
//                 }),
//                 &BLUE,
//             ))
//             .unwrap();

//         chart
//             .draw_series(LineSeries::new(
//                 (0..=num_samples).map(|i| {
//                     let s = limiter.process(sine_sample(i));
//                     (i as f32, s)
//                 }),
//                 &RED,
//             ))
//             .unwrap();

//         root.present().unwrap();
//     }
// }
