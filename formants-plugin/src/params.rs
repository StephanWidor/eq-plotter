use std::sync::atomic;

use super::*;
use audio_lib::*;
use nice_plug::params::Params;
use std::ops::RangeInclusive;

const SMOOTHING_LENGTH: f32 = 20_f32;

struct FrequencyBounds {
    pub matrix: nalgebra::Matrix2x4<f32>,
    pub f_ranges: [RangeInclusive<f32>; 2],
}

impl Default for FrequencyBounds {
    fn default() -> Self {
        let matrix: nalgebra::Matrix2x4<f32> = nalgebra::matrix![
        320_f32, 1000_f32, 600_f32,300_f32;
        800_f32, 1400_f32, 2000_f32,3000_f32];
        let f_ranges: [RangeInclusive<f32>; 2] =
            std::array::from_fn(|i| matrix.row(i).min()..=matrix.row(i).max());
        Self {
            matrix: matrix,
            f_ranges: f_ranges,
        }
    }
}

impl FrequencyBounds {
    pub fn frequency_to_normalized(&self, frequency: f32, index: usize) -> f32 {
        assert!(index < 2);
        ((frequency - self.f_ranges[index].start()) / range::get_length(&self.f_ranges[index]))
            .clamp(0_f32, 1_f32)
    }
}

#[derive(nice::Params)]
pub struct FormantParams {
    #[id = "Frequency"]
    pub normalized_frequency: nice::FloatParam,

    #[id = "Q"]
    pub q: nice::FloatParam,
}

impl FormantParams {
    pub fn new(default_q: f32, names_suffix: &str) -> Self {
        Self {
            normalized_frequency: nice::FloatParam::new(
                format!("Frequency{names_suffix}"),
                0.5_f32,
                nice::FloatRange::Linear {
                    min: 0_f32,
                    max: 1_f32,
                },
            )
            .with_smoother(nice::SmoothingStyle::Linear(SMOOTHING_LENGTH)),
            q: nice::FloatParam::new(
                format!("Q{names_suffix}"),
                default_q,
                nice::FloatRange::Linear {
                    min: 1_f32,
                    max: 20_f32,
                },
            )
            .with_smoother(nice::SmoothingStyle::Linear(SMOOTHING_LENGTH)),
        }
    }

    pub fn set_normalized_frequency(
        &self,
        normalized_frequency: f32,
        setter: &nice::ParamSetter<'_>,
    ) {
        setter.begin_set_parameter(&self.normalized_frequency);
        setter.set_parameter(&self.normalized_frequency, normalized_frequency);
        setter.end_set_parameter(&self.normalized_frequency);
    }

    pub fn set_q(&self, q: f32, setter: &nice::ParamSetter<'_>) {
        setter.begin_set_parameter(&self.q);
        setter.set_parameter(&self.q, q);
        setter.end_set_parameter(&self.q);
    }
}

#[derive(nice::Params)]
pub struct PluginParams {
    #[persist = "editor_state"]
    pub editor_state: std::sync::Arc<nice_plug_egui::EguiState>,

    #[nested(array, group = "Formant")]
    pub formants: [FormantParams; 2],

    #[id = "Dry Gain (dB)"]
    pub dry_gain_db: nice::FloatParam,

    pub sample_rate: nice::AtomicF32,

    frequency_bounds: FrequencyBounds,
}

impl PluginParams {
    pub fn new() -> Self {
        Self {
            editor_state: nice_plug_egui::EguiState::from_size(420, 500),
            formants: [
                FormantParams::new(10_f32, "[1]"),
                FormantParams::new(20_f32, "[2]"),
            ],
            dry_gain_db: nice::FloatParam::new(
                format!("Mix"),
                -12_f32,
                nice::FloatRange::Linear {
                    min: -32_f32,
                    max: 0_f32,
                },
            )
            .with_smoother(nice::SmoothingStyle::Linear(SMOOTHING_LENGTH)),
            sample_rate: nice::AtomicF32::new(48000_f32),
            frequency_bounds: FrequencyBounds::default(),
        }
    }

    pub fn set_dry_gain_db(&self, gain_db: f32, setter: &nice::ParamSetter<'_>) {
        setter.begin_set_parameter(&self.dry_gain_db);
        setter.set_parameter(&self.dry_gain_db, gain_db);
        setter.end_set_parameter(&self.dry_gain_db);
    }

    pub fn get_biquad_coefficients(&self) -> [biquad::coefficients::Coefficients<f32>; 3] {
        use biquad::coefficients::Coefficients;
        let frequencies = self.get_frequencies();
        let sample_rate = self.sample_rate.load(atomic::Ordering::Relaxed);
        [
            Self::get_coefficients(
                frequencies[0],
                self.formants[0].q.value(),
                self.gain_db_for_frequency(frequencies[0], 0),
                sample_rate,
            ),
            Self::get_coefficients(
                frequencies[1],
                self.formants[1].q.value(),
                self.gain_db_for_frequency(frequencies[1], 1),
                sample_rate,
            ),
            Coefficients::from_volume_db(self.dry_gain_db.value()),
        ]
    }

    fn get_frequencies(&self) -> nalgebra::Vector2<f32> {
        let f: [f32; 2] = std::array::from_fn(|i| self.formants[i].normalized_frequency.value());
        let v = nalgebra::vector![
            (1_f32 - f[0]) * (1_f32 - f[1]),
            f[0] * (1_f32 - f[1]),
            f[0] * f[1],
            (1_f32 - f[0]) * f[1]
        ];
        self.frequency_bounds.matrix * v
    }

    fn gain_db_for_frequency(&self, frequency: f32, index: usize) -> f32 {
        let bounds_normalized_frequency = self
            .frequency_bounds
            .frequency_to_normalized(frequency, index);
        if index == 0 {
            12_f32 - bounds_normalized_frequency * 6_f32
        } else {
            6_f32 + bounds_normalized_frequency * 6_f32
        }
    }

    fn get_coefficients(
        frequency: f32,
        q: f32,
        gain_db: f32,
        sample_rate: f32,
    ) -> biquad::coefficients::Coefficients<f32> {
        let mut c = biquad::coefficients::Coefficients::from_bandpass(frequency, q, sample_rate);
        c.add_makeup_gain(eq::Gain::Db(gain_db));
        c
    }
}
