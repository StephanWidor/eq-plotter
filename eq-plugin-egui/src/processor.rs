use crate::*;
use audio_lib::*;
use std::sync::{self, atomic};

type FilterCoefficients<const NUM_BANDS: usize> =
    [biquad::coefficients::Coefficients<f32>; NUM_BANDS];
type Filters<const NUM_BANDS: usize> = [biquad::filter::State<f32>; NUM_BANDS];

pub struct Processor<
    const NUM_BANDS: usize,
    const NUM_CHANNELS: usize,
    const ANALYZER_NUM_BINS: usize,
> {
    plugin_params: sync::Arc<params::PluginParams<NUM_BANDS, NUM_CHANNELS, ANALYZER_NUM_BINS>>,
    eqs: [eq::Eq<f32>; NUM_BANDS],
    coefficients: FilterCoefficients<NUM_BANDS>,
    filters: [Filters<NUM_BANDS>; NUM_CHANNELS],
}

impl<const NUM_BANDS: usize, const NUM_CHANNELS: usize, const ANALYZER_NUM_BINS: usize>
    Processor<NUM_BANDS, NUM_CHANNELS, ANALYZER_NUM_BINS>
{
    pub fn new(
        plugin_params: sync::Arc<params::PluginParams<NUM_BANDS, NUM_CHANNELS, ANALYZER_NUM_BINS>>,
    ) -> Self {
        Self {
            plugin_params: plugin_params,
            eqs: [Self::INIT_EQ; NUM_BANDS],
            coefficients: [Self::INIT_FILTER_COEFFICIENTS; NUM_BANDS],
            filters: std::array::from_fn(|_| std::array::from_fn(|_| biquad::filter::State::new())),
        }
    }

    pub fn initialize(&mut self) -> bool {
        for channel_filters in self.filters.iter_mut() {
            for filter in channel_filters.iter_mut() {
                filter.reset();
            }
        }
        self.update_coefficients(
            &self.plugin_params.eqs(),
            self.plugin_params
                .sample_rate
                .load(atomic::Ordering::Relaxed),
        )
    }

    pub fn process(&mut self, buffer: &mut nice::Buffer) {
        self.update_coefficients(
            &self.plugin_params.eqs(),
            self.plugin_params
                .sample_rate
                .load(atomic::Ordering::Relaxed),
        );

        assert!(buffer.channels() <= NUM_CHANNELS);
        let buffer_slice = buffer.as_slice();
        for channel in 0..buffer_slice.len() {
            let channel_samples = buffer_slice.get_mut(channel).unwrap();
            let channel_filters = &mut self.filters[channel];
            for sample in (*channel_samples).iter_mut() {
                let mut processing_sample = *sample;
                for i in 0..NUM_BANDS {
                    processing_sample =
                        channel_filters[i].process(&self.coefficients[i], processing_sample);
                }
                *sample = processing_sample;
            }
        }
    }

    fn update_coefficients(&mut self, new_eqs: &[eq::Eq<f32>], sample_rate: f32) -> bool {
        assert!(new_eqs.len() >= NUM_BANDS);
        let mut success = true;
        for i in 0..NUM_BANDS {
            let eq = &mut self.eqs[i];
            let new_eq = &new_eqs[i];
            if *new_eq != *eq {
                let new_coefficients =
                    biquad::coefficients::Coefficients::from_eq(new_eq, sample_rate);
                if !biquad::utils::is_stable(&new_coefficients) {
                    success = false;
                } else {
                    *eq = new_eq.clone();
                    self.coefficients[i] = new_coefficients;
                }
            }
        }
        success
    }

    const INIT_FILTER_COEFFICIENTS: biquad::coefficients::Coefficients<f32> =
        biquad::coefficients::Coefficients::muted();
    const INIT_EQ: eq::Eq<f32> = eq::Eq {
        gain: eq::Gain::Amplitude(0_f32),
        frequency: eq::Frequency::Hz(0_f32),
        q: 0.0,
        eq_type: eq::EqType::Volume,
    };
}
