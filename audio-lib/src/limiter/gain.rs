use super::*;

pub trait Knee<F: utils::Float> {
    type Params;
    fn new(params: Self::Params) -> Self;
    fn new_inactive() -> Self;
    fn set(&mut self, params: Self::Params);
    fn gain(&self, sample: F) -> F;
}

#[derive(Clone, Copy, Debug)]
pub struct HardKneeParams<F: utils::Float> {
    threshold_linear: F,
    compression_factor: F,
}

#[derive(Clone, Copy, Debug)]
pub struct HardKnee<F: utils::Float> {
    params: HardKneeParams<F>,
}

impl<F: utils::Float> Knee<F> for HardKnee<F> {
    type Params = HardKneeParams<F>;

    fn new(params: Self::Params) -> Self {
        Self { params: params }
    }

    fn new_inactive() -> Self {
        Self {
            params: HardKneeParams {
                threshold_linear: F::infinity(),
                compression_factor: F::ONE,
            },
        }
    }

    fn set(&mut self, params: Self::Params) {
        self.params = params;
    }

    fn gain(&self, sample: F) -> F {
        assert!(sample >= F::ZERO);
        if sample <= self.params.threshold_linear {
            F::ONE
        } else {
            (self.params.threshold_linear
                + self.params.compression_factor * (sample - self.params.threshold_linear))
                / sample
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SoftKneeParams<F: utils::Float> {
    pub threshold_linear: F,
    pub compression_factor: F,
    pub knee_ratio: F,
}

#[derive(Clone, Copy, Debug)]
pub struct SoftKnee<F: utils::Float> {
    threshold_linear: F,
    compression_factor: F,
    knee_bounds: [F; 2],
    knee_bounds_compressed: [F; 2],
}

impl<F: utils::Float> Knee<F> for SoftKnee<F> {
    type Params = SoftKneeParams<F>;

    fn new(params: Self::Params) -> Self {
        assert!(params.knee_ratio > F::ONE);
        let knee_bounds = [
            params.threshold_linear / params.knee_ratio,
            params.threshold_linear * params.knee_ratio,
        ];
        Self {
            threshold_linear: params.threshold_linear,
            compression_factor: params.compression_factor,
            knee_bounds: knee_bounds,
            knee_bounds_compressed: std::array::from_fn(|i| {
                Self::full_compress(
                    knee_bounds[i],
                    params.threshold_linear,
                    params.compression_factor,
                )
            }),
        }
    }

    fn new_inactive() -> Self {
        Self {
            threshold_linear: F::infinity(),
            compression_factor: F::ONE,
            knee_bounds: [F::infinity(), F::infinity()],
            knee_bounds_compressed: [F::ONE, F::ONE],
        }
    }

    fn set(&mut self, params: Self::Params) {
        self.threshold_linear = params.threshold_linear;
        self.compression_factor = params.compression_factor;
        self.knee_bounds = [
            params.threshold_linear / params.knee_ratio,
            params.threshold_linear * params.knee_ratio,
        ];
        self.knee_bounds_compressed = std::array::from_fn(|i| {
            Self::full_compress(
                self.knee_bounds[i],
                params.threshold_linear,
                params.compression_factor,
            )
        });
    }

    fn gain(&self, sample: F) -> F {
        assert!(sample >= F::ZERO);
        if sample <= self.knee_bounds[0] {
            F::ONE
        } else if sample < self.knee_bounds[1] {
            let knee_length = self.knee_bounds[1] - self.knee_bounds[0];
            let ratio = (sample - self.knee_bounds[0]) / knee_length;
            let t1_offset = self.knee_bounds[0]
                + ratio * (self.knee_bounds_compressed[0] - self.knee_bounds[0]);
            (t1_offset + ratio * (self.knee_bounds_compressed[1] - t1_offset)) / sample
        } else {
            (Self::full_compress(sample, self.threshold_linear, self.compression_factor)) / sample
        }
    }
}

impl<F: utils::Float> SoftKnee<F> {
    fn full_compress(sample: F, threshold: F, compression_factor: F) -> F {
        threshold + compression_factor * (sample - threshold)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_plot() {
//         let knee = SoftKnee::new(SoftKneeParams {
//             threshold_linear: 0.9,
//             compression_factor: 0.3,
//             knee_ratio: 1.5,
//         });

//         use plotters::prelude::*;
//         let root = BitMapBackend::new("/tmp/soft_knee.png", (2048, 2048)).into_drawing_area();
//         root.fill(&WHITE).unwrap();
//         let mut chart = ChartBuilder::on(&root)
//             .x_label_area_size(300)
//             .y_label_area_size(300)
//             .build_cartesian_2d(0f32..2f32, 0f32..2f32)
//             .unwrap();

//         chart.configure_mesh().draw().unwrap();
//         let num_line_points = 1000;

//         chart
//             .draw_series(LineSeries::new(
//                 (0..=(2 * num_line_points)).map(|i| {
//                     let s = (i as f32) / (num_line_points as f32);
//                     (s, s * knee.gain(s))
//                 }),
//                 &BLUE,
//             ))
//             .unwrap();
//         chart
//             .draw_series(LineSeries::new(
//                 (0..=(2 * num_line_points)).map(|i| {
//                     let s = (i as f32) / (num_line_points as f32);
//                     (s, knee.gain(s))
//                 }),
//                 &RED,
//             ))
//             .unwrap();

//         root.present().unwrap();
//     }
// }
