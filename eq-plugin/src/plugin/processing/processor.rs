use crate::plugin::*;
use audio_lib::*;
use std::sync::{self, atomic};

type Filter = biquad::filter::Filter<f32>;
type MultibandFilter<const NUM_BANDS: usize> = [Filter; NUM_BANDS];

pub struct Processor<
    const NUM_BANDS: usize,
    const NUM_CHANNELS: usize,
    const ANALYZER_NUM_BINS: usize,
> {
    plugin_params: sync::Arc<params::PluginParams<NUM_BANDS, NUM_CHANNELS, ANALYZER_NUM_BINS>>,
    multiband_eq: eq::MultibandEq<f32, NUM_BANDS>,
    channel_filters: [MultibandFilter<NUM_BANDS>; NUM_CHANNELS],
}

impl<const NUM_BANDS: usize, const NUM_CHANNELS: usize, const ANALYZER_NUM_BINS: usize>
    Processor<NUM_BANDS, NUM_CHANNELS, ANALYZER_NUM_BINS>
{
    pub fn new(
        plugin_params: sync::Arc<params::PluginParams<NUM_BANDS, NUM_CHANNELS, ANALYZER_NUM_BINS>>,
    ) -> Self {
        Self {
            plugin_params: plugin_params,
            multiband_eq: eq::MultibandEq {
                eqs: [Self::INIT_EQ; NUM_BANDS],
                processing_type: Self::INIT_PROCESSING_TYPE,
            },
            channel_filters: std::array::from_fn(|_| {
                std::array::from_fn(|_| Filter::new(Self::INIT_FILTER_COEFFICIENTS))
            }),
        }
    }

    pub fn initialize(&mut self) -> bool {
        self.update(
            self.plugin_params.to_multiband_eq(),
            self.plugin_params
                .sample_rate
                .load(atomic::Ordering::Relaxed),
        );
        for filters in self.channel_filters.iter_mut() {
            for filter in filters.iter_mut() {
                filter.reset_state();
            }
        }
        true
    }

    pub fn process(&mut self, buffer: &mut nice::Buffer) {
        self.update(
            self.plugin_params.to_multiband_eq(),
            self.plugin_params
                .sample_rate
                .load(atomic::Ordering::Relaxed),
        );

        // TODO: maybe the processing order is not optimal, creating the filter iterator for each channel and sample again.
        assert!(buffer.channels() <= NUM_CHANNELS);
        let buffer_slice = buffer.as_slice();
        match self.multiband_eq.processing_type {
            eq::MultibandType::Sequential => {
                for channel in 0..buffer_slice.len() {
                    let channel_samples = buffer_slice.get_mut(channel).unwrap();
                    for sample in (*channel_samples).iter_mut() {
                        *sample = biquad::multiband::sequential::process(
                            self.active_channel_filters(channel),
                            *sample,
                        );
                    }
                }
            }
            eq::MultibandType::ParallelSum => {
                for channel in 0..buffer_slice.len() {
                    let channel_samples = buffer_slice.get_mut(channel).unwrap();
                    for sample in (*channel_samples).iter_mut() {
                        *sample = biquad::multiband::parallel_sum::process(
                            self.active_channel_filters(channel),
                            *sample,
                        );
                    }
                }
            }
            eq::MultibandType::ParallelAverage => {
                for channel in 0..buffer_slice.len() {
                    let channel_samples = buffer_slice.get_mut(channel).unwrap();
                    for sample in (*channel_samples).iter_mut() {
                        *sample = biquad::multiband::parallel_average::process(
                            self.active_channel_filters(channel),
                            *sample,
                        );
                    }
                }
            }
        }
    }

    // TODO: this doesn't work :-(
    // fn process_with_sample_processor<
    //     SampleProcessor: FnMut(dyn Iterator<Item = &mut Filter>, f32) -> f32,
    // >(
    //     &mut self,
    //     buffer: &mut nice::Buffer,
    //     sample_processor: &SampleProcessor,
    // ) {
    //     let buffer_slice = buffer.as_slice();
    //     for channel in 0..buffer_slice.len() {
    //         let channel_samples = buffer_slice.get_mut(channel).unwrap();
    //         for sample in (*channel_samples).iter_mut() {
    //             *sample = sample_processor(self.active_channel_filters(channel), *sample);
    //         }
    //     }
    // }

    fn active_channel_filters(&mut self, channel: usize) -> impl Iterator<Item = &mut Filter> {
        self.channel_filters[channel]
            .iter_mut()
            .zip(
                self.multiband_eq
                    .eqs
                    .iter()
                    .map(|eq| eq.eq_type.is_active()),
            )
            .filter(|(_, active)| *active)
            .map(|(filter, _)| filter)
    }

    fn update(&mut self, new_multiband_eq: eq::MultibandEq<f32, NUM_BANDS>, sample_rate: f32) {
        self.multiband_eq.processing_type = new_multiband_eq.processing_type;
        for i in 0..NUM_BANDS {
            let eq = &mut self.multiband_eq.eqs[i];
            let new_eq = &new_multiband_eq.eqs[i];
            if *new_eq != *eq {
                let new_coefficients =
                    biquad::coefficients::Coefficients::from_eq(new_eq, sample_rate);
                if biquad::is_stable(&new_coefficients) {
                    *eq = new_eq.clone();
                    for channel_filters in self.channel_filters.iter_mut() {
                        channel_filters[i].set_coefficients(new_coefficients.clone(), false);
                    }
                }
            }
        }
    }

    const INIT_FILTER_COEFFICIENTS: biquad::coefficients::Coefficients<f32> =
        biquad::coefficients::Coefficients::muted();
    const INIT_EQ: eq::Eq<f32> = eq::Eq {
        gain: eq::Gain::Amplitude(0_f32),
        frequency: eq::Frequency::Hz(0_f32),
        q: 0.0,
        eq_type: eq::EqType::Volume,
        makeup_gain: eq::Gain::Amplitude(0_f32),
    };
    const INIT_PROCESSING_TYPE: eq::MultibandType = eq::MultibandType::Sequential;
}
