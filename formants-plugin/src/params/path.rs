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
    pub control_points: [Point; NUM_CONTROL_POINTS],
}

impl Path {
    pub fn new() -> Self {
        Self {
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
        }
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

    pub fn bezier_matrix(&self) -> Matrix2x4<f32> {
        let control_points: [[f32; 2]; NUM_CONTROL_POINTS] =
            std::array::from_fn(|col| self.control_points[col].load_xy::<f32>());

        Matrix2x4::<f32>::from_fn(|r, c| control_points[c][r]) * Self::BEZIER_BASE
    }

    pub fn get_xy<F: audio_lib::utils::Float>(&self) -> [F; 2] {
        let matrix = self.bezier_matrix();
        let t = self.t.value();
        [
            F::from_float(Self::calc_horner(&matrix.row(0), t)),
            F::from_float(Self::calc_horner(&matrix.row(1), t)),
        ]
    }

    pub fn calc_horner(
        row: &Matrix<
            f32,
            Const<1>,
            Const<4>,
            ViewStorage<'_, f32, Const<1>, Const<4>, Const<1>, Const<2>>,
        >,
        t: f32,
    ) -> f32 {
        let n = row.ncols();
        let mut accumulator = row[n - 1];
        for j in 2..=n {
            accumulator *= t;
            accumulator += row[n - j];
        }
        accumulator
    }

    pub const BEZIER_BASE: Matrix4<f32> = nalgebra::matrix![
        1.0, -3.0, 3.0, -1.0; //
        0.0, 3.0, -6.0, 3.0; //
        0.0, 0.0, 3.0, -3.0; //
        0.0, 0.0, 0.0, 1.0; //
    ];
}
