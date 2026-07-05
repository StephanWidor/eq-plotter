use std::sync::atomic;

use super::*;

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
    pub control_points: [Point; 2],
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
            control_points: [Point::new(0_f32, 0_f32), Point::new(1_f32, 1_f32)],
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

    pub fn get_xy<F: audio_lib::utils::Float>(&self) -> [F; 2] {
        let t = F::from_float(self.t.value());
        let one_minus_t = F::ONE - t;
        let start = self.control_points[0].load_xy::<F>();
        let end = self.control_points[1].load_xy::<F>();
        [
            F::from_float(one_minus_t * start[0] + t * end[0]),
            F::from_float(one_minus_t * start[1] + t * end[1]),
        ]
    }
}
