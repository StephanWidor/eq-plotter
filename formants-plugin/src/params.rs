use std::sync::atomic;

use super::*;
use audio_lib::*;
use nice_plug::params::Params;

const SMOOTHING_LENGTH: f32 = 20_f32;

const FREQUENCY_BOUNDS: nalgebra::Matrix2x4<f32> = nalgebra::matrix![
        320_f32, 1000_f32, 1000_f32,300_f32;
        800_f32, 1400_f32, 1600_f32,3000_f32];

#[derive(nice::Params)]
pub struct FormantParams {
    #[id = "Frequency"]
    pub normalized_frequency: nice::FloatParam,

    #[id = "Gain (dB)"]
    pub gain_db: nice::FloatParam,
}

impl FormantParams {
    pub fn new(names_suffix: &str) -> Self {
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
            gain_db: nice::FloatParam::new(
                format!("Gain (dB){names_suffix}"),
                6_f32,
                nice::FloatRange::Linear {
                    min: -100_f32,
                    max: 6_f32,
                },
            )
            .with_smoother(nice::SmoothingStyle::Linear(SMOOTHING_LENGTH)),
        }
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
}

impl PluginParams {
    pub fn new() -> Self {
        Self {
            editor_state: nice_plug_egui::EguiState::from_size(1000, 700),
            formants: [FormantParams::new("[1]"), FormantParams::new("[2]")],
            dry_gain_db: nice::FloatParam::new(
                format!("Mix"),
                -12_f32,
                nice::FloatRange::Linear {
                    min: -100_f32,
                    max: 6_f32,
                },
            )
            .with_smoother(nice::SmoothingStyle::Linear(SMOOTHING_LENGTH)),
            sample_rate: nice::AtomicF32::new(48000_f32),
        }
    }

    pub fn get_biquad_coefficients(&self) -> [biquad::coefficients::Coefficients<f32>; 3] {
        use biquad::coefficients::Coefficients;
        let frequencies = self.get_frequencies();
        let sample_rate = self.sample_rate.load(atomic::Ordering::Relaxed);
        [
            Self::get_coefficients(frequencies[0], 10_f32, sample_rate),
            Self::get_coefficients(frequencies[1], 10_f32, sample_rate),
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
        FREQUENCY_BOUNDS * v
    }

    fn get_coefficients(
        frequency: f32,
        gain_db: f32,
        sample_rate: f32,
    ) -> biquad::coefficients::Coefficients<f32> {
        let mut c =
            biquad::coefficients::Coefficients::from_bandpass(frequency, 10_f32, sample_rate);
        c.add_makeup_gain(eq::Gain::Db(gain_db));
        c
    }
}
