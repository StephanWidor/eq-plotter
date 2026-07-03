use super::*;
use audio_lib::*;
use nice_plug::params::Params;
use std::sync::atomic;

const SMOOTHING_LENGTH: f32 = 20_f32;

#[derive(nice::Params)]
pub struct PluginParams {
    #[persist = "editor_state"]
    pub editor_state: std::sync::Arc<nice_plug_egui::EguiState>,

    #[id = "x"]
    pub x: nice::FloatParam,

    #[id = "y"]
    pub y: nice::FloatParam,

    #[id = "Dry Gain (dB)"]
    pub dry_gain_db: nice::FloatParam,

    pub sample_rate: nice::AtomicF32,

    pub frequency_transform: FrequencyTransform,
}

impl PluginParams {
    pub fn new() -> Self {
        Self {
            editor_state: nice_plug_egui::EguiState::from_size(450, 500),
            x: nice::FloatParam::new(
                "x",
                0.5_f32,
                nice::FloatRange::Linear {
                    min: 0_f32,
                    max: 1_f32,
                },
            )
            .with_smoother(nice::SmoothingStyle::Linear(SMOOTHING_LENGTH)),
            y: nice::FloatParam::new(
                "y",
                0.5_f32,
                nice::FloatRange::Linear {
                    min: 0_f32,
                    max: 1_f32,
                },
            )
            .with_smoother(nice::SmoothingStyle::Linear(SMOOTHING_LENGTH)),
            dry_gain_db: nice::FloatParam::new(
                format!("Mix"),
                -36_f32,
                nice::FloatRange::Linear {
                    min: -60_f32,
                    max: 0_f32,
                },
            )
            .with_smoother(nice::SmoothingStyle::Linear(SMOOTHING_LENGTH)),
            sample_rate: nice::AtomicF32::new(48000_f32),
            frequency_transform: FrequencyTransform::default(),
        }
    }

    pub fn set_x(&self, x: f32, setter: &nice::ParamSetter<'_>) {
        setter.begin_set_parameter(&self.x);
        setter.set_parameter(&self.x, x);
        setter.end_set_parameter(&self.x);
    }

    pub fn set_y(&self, y: f32, setter: &nice::ParamSetter<'_>) {
        setter.begin_set_parameter(&self.y);
        setter.set_parameter(&self.y, y);
        setter.end_set_parameter(&self.y);
    }

    pub fn set_dry_gain_db(&self, gain_db: f32, setter: &nice::ParamSetter<'_>) {
        setter.begin_set_parameter(&self.dry_gain_db);
        setter.set_parameter(&self.dry_gain_db, gain_db);
        setter.end_set_parameter(&self.dry_gain_db);
    }

    pub fn get_multiband_coefficients(&self) -> [biquad::coefficients::Coefficients<f32>; 3] {
        use biquad::coefficients::Coefficients;
        let frequencies = self.get_frequencies();
        let sample_rate = self.sample_rate.load(atomic::Ordering::Relaxed);
        [
            Self::get_single_band_coefficients(
                frequencies[0],
                self.frequency_transform.q_for_frequency(frequencies[0], 0),
                self.frequency_transform
                    .gain_db_for_frequency(frequencies[0], 0),
                sample_rate,
            ),
            Self::get_single_band_coefficients(
                frequencies[1],
                self.frequency_transform.q_for_frequency(frequencies[1], 1),
                self.frequency_transform
                    .gain_db_for_frequency(frequencies[1], 1),
                sample_rate,
            ),
            Coefficients::from_volume_db(self.dry_gain_db.value()),
        ]
    }

    fn get_frequencies(&self) -> [f32; 2] {
        self.frequency_transform
            .coordinates_to_frequencies(self.x.value(), self.y.value())
    }

    fn get_single_band_coefficients(
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
