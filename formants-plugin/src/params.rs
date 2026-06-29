use std::sync::atomic;

use super::*;
use audio_lib::*;
use nice_plug::params::Params;
use std::ops::RangeInclusive;

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

#[derive(nice::Params)]
pub struct FormantParams {
    #[id = "Frequency"]
    pub normalized_frequency: nice::FloatParam,
}

/// For transforming/distorting coordinates (x,y) in [0;1]^2 into formant frequencies
pub struct FrequencyTransform {
    pub bounds_matrix: nalgebra::Matrix2x4<f32>,
    pub frequency_ranges: [RangeInclusive<f32>; 2],
}

impl Default for FrequencyTransform {
    fn default() -> Self {
        let bounds_matrix: nalgebra::Matrix2x4<f32> = nalgebra::matrix![
            300_f32, 1000_f32, 1000_f32,  260_f32;
            800_f32, 1400_f32, 2000_f32, 3200_f32];
        let f_ranges: [RangeInclusive<f32>; 2] =
            std::array::from_fn(|i| bounds_matrix.row(i).min()..=bounds_matrix.row(i).max());
        Self {
            bounds_matrix: bounds_matrix,
            frequency_ranges: f_ranges,
        }
    }
}

impl FrequencyTransform {
    pub fn coordinates_to_frequencies(&self, x: f32, y: f32) -> [f32; 2] {
        let xy = x * y;
        let v = self.bounds_matrix * nalgebra::vector![1_f32 - x - y + xy, x - xy, xy, y - xy];
        [v[0], v[1]]
    }

    pub fn gain_db_for_frequency(&self, frequency: f32, index: usize) -> f32 {
        let ratio = self.frequency_ratio(frequency, index);
        if index == 0 {
            12_f32 - ratio * 6_f32
        } else {
            6_f32 + ratio * 6_f32
        }
    }

    pub fn q_for_frequency(&self, frequency: f32, index: usize) -> f32 {
        if index == 0 {
            10_f32
        } else {
            8_f32 + self.frequency_ratio(frequency, index) * 10_f32
        }
    }

    fn frequency_ratio(&self, frequency: f32, index: usize) -> f32 {
        assert!(index < 2);
        ((frequency - self.frequency_ranges[index].start())
            / range::get_length(&self.frequency_ranges[index]))
        .clamp(0_f32, 1_f32)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_bounds() {
        let bound_coords = [
            [0_f32, 0_f32],
            [1_f32, 0_f32],
            [1_f32, 1_f32],
            [0_f32, 1_f32],
        ];
        let transform = FrequencyTransform::default();
        for i in 0..4 {
            let [x, y] = bound_coords[i];
            let bound = transform.bounds_matrix.column(i);
            let frequencies = transform.coordinates_to_frequencies(x, y);
            for j in 0..2 {
                assert_approx_eq!(bound[j], frequencies[j]);
            }
        }
    }
}
