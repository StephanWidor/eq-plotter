use nalgebra::*;
use std::sync::atomic;

use super::*;

pub const NUM_CONTROL_POINTS: usize = 4;

#[derive(nice::Params)]
pub struct Point {
    #[persist = "x"]
    pub x: nice::AtomicF32,

    #[persist = "y"]
    pub y: nice::AtomicF32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x: nice::AtomicF32::new(x),
            y: nice::AtomicF32::new(y),
        }
    }

    pub fn load_xy<F: audio_lib::utils::Float>(&self) -> [F; 2] {
        [
            F::from_float(self.x.load(atomic::Ordering::Relaxed)),
            F::from_float(self.y.load(atomic::Ordering::Relaxed)),
        ]
    }

    pub fn store_xy(&self, x: f32, y: f32) {
        self.x
            .store(x.clamp(0_f32, 1_f32), atomic::Ordering::Relaxed);
        self.y
            .store(y.clamp(0_f32, 1_f32), atomic::Ordering::Relaxed);
    }
}

#[derive(nice::Params)]
pub struct Path {
    #[id = "Enabled"]
    pub enabled: nice::BoolParam,

    #[id = "t"]
    pub t: nice::FloatParam,

    #[nested(array, group = "control_points")]
    control_points: [Point; NUM_CONTROL_POINTS],

    spline_base: Matrix4<f32>,
    spline_coefficients: [[nice::AtomicF32; 4]; 2],
}

impl Path {
    pub fn new() -> Self {
        let path = Self {
            enabled: nice::BoolParam::new("Enabled", false),
            t: nice::FloatParam::new(
                "t",
                0.5_f32,
                nice::FloatRange::Linear {
                    min: 0_f32,
                    max: 1_f32,
                },
            ),
            control_points: [
                Point::new(0.1_f32, 0.6_f32),
                Point::new(0.2_f32, 0_f32),
                Point::new(0.5_f32, 0.8_f32),
                Point::new(1_f32, 0.6_f32),
            ],
            spline_base: (1_f32 / 3_f32)
                * nalgebra::matrix![
                    3_f32, -19_f32, 32_f32, -16_f32;
                    0_f32, 24_f32, -56_f32, 32_f32;
                    0_f32, -8_f32, 40_f32, -32_f32;
                    0_f32, 3_f32, -16_f32, 16_f32],
            spline_coefficients: std::array::from_fn(|_| {
                std::array::from_fn(|_| nice::AtomicF32::new(0_f32))
            }),
        };
        path.update_coefficients();
        path
    }

    pub fn set_enabled(&self, enabled: bool, setter: &nice::ParamSetter<'_>) {
        setter.begin_set_parameter(&self.enabled);
        setter.set_parameter(&self.enabled, enabled);
        setter.end_set_parameter(&self.enabled);
    }

    pub fn set_t(&self, t: f32, setter: &nice::ParamSetter<'_>) {
        setter.begin_set_parameter(&self.t);
        setter.set_parameter(&self.t, t);
        setter.end_set_parameter(&self.t);
    }

    pub fn set_control_point(&self, x: f32, y: f32, i: usize) {
        assert!(i < NUM_CONTROL_POINTS);
        self.control_points[i].store_xy(x, y);
        self.update_coefficients();
    }

    pub fn get_control_point<F: audio_lib::utils::Float>(&self, i: usize) -> [F; 2] {
        assert!(i < NUM_CONTROL_POINTS);
        self.control_points[i].load_xy()
    }

    pub fn get_point<F: audio_lib::utils::Float>(&self) -> [F; 2] {
        self.get_point_for_t(self.t.value())
    }

    pub fn get_point_for_t<F: audio_lib::utils::Float>(&self, t: f32) -> [F; 2] {
        [
            F::from_float(Self::calc_horner(&self.get_coefficients(0), t)),
            F::from_float(Self::calc_horner(&self.get_coefficients(1), t)),
        ]
    }

    /// coefficients[0] + coefficients[1] * t + coefficients[2] * t^2 + ... + coefficients[N-1] * t^(N-1)
    /// (calculated via horner scheme)
    pub fn calc_horner<const N: usize>(coefficients: &[f32; N], t: f32) -> f32 {
        let mut accumulator = coefficients[N - 1];
        for j in 2..=N {
            accumulator = accumulator * t + coefficients[N - j];
        }
        accumulator
    }

    pub fn get_coefficients(&self, i: usize) -> [f32; NUM_CONTROL_POINTS] {
        assert!(i < 2);
        let c = &self.spline_coefficients[i];
        std::array::from_fn(|j| c[j].load(atomic::Ordering::Relaxed))
    }

    fn update_coefficients(&self) {
        for i in 0..2 {
            let c = &self.spline_coefficients[i];
            for j in 0..4 {
                c[j].store(
                    (self.geometry_row(i) * self.spline_base.column(j))[0],
                    atomic::Ordering::Relaxed,
                );
            }
        }
    }

    fn geometry_row(&self, i: usize) -> SMatrix<f32, 1, NUM_CONTROL_POINTS> {
        assert!(i < 2);
        if i == 0 {
            SMatrix::from_fn(|_, j| self.control_points[j].x.load(atomic::Ordering::Relaxed))
        } else {
            SMatrix::from_fn(|_, j| self.control_points[j].y.load(atomic::Ordering::Relaxed))
        }
    }
}
