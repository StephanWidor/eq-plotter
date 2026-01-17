use crate::*;
use audio_lib::*;
use nih_plug::prelude as nih;
use std::sync;

type ChannelFilters = [biquad::filter::Filter<f32>; params::PluginParams::NUM_BANDS];

pub struct Processor {
    eqs: [eq::Eq<f32>; params::PluginParams::NUM_BANDS],
    filters: [ChannelFilters; 2],
}

impl Default for Processor {
    fn default() -> Self {
        Self {
            eqs: [Self::INIT_EQ; params::PluginParams::NUM_BANDS],
            filters: [Self::init_channel_filters(), Self::init_channel_filters()],
        }
    }
}

impl Processor {
    const INIT_FILTER_COEFFICIENTS: biquad::coefficients::Coefficients<f32> =
        biquad::coefficients::Coefficients::muted();
    const INIT_EQ: eq::Eq<f32> = eq::Eq {
        gain: eq::Gain::Db(std::f32::NEG_INFINITY),
        frequency: eq::Frequency::LogHz(std::f32::NEG_INFINITY),
        q: 0.0,
        eq_type: eq::EqType::Volume,
    };

    fn init_channel_filters() -> ChannelFilters {
        array_init::array_init(|_| biquad::filter::Filter::new(&Self::INIT_FILTER_COEFFICIENTS))
    }

    pub fn initialize(&mut self, params: sync::Arc<params::PluginParams>) -> bool {
        self.update_filters(
            &params.eqs(),
            params.sample_rate.load(sync::atomic::Ordering::Relaxed),
            true,
        )
    }

    fn update_filters(
        &mut self,
        new_eqs: &[eq::Eq<f32>],
        sample_rate: f32,
        reset_state: bool,
    ) -> bool {
        assert!(new_eqs.len() >= params::PluginParams::NUM_BANDS);
        let mut success = true;
        for i in 0..params::PluginParams::NUM_BANDS {
            let eq = &mut self.eqs[i];
            let new_eq = &new_eqs[i];
            if *new_eq != *eq {
                let new_coefficients =
                    biquad::coefficients::Coefficients::from_eq(new_eq, sample_rate);
                if !biquad::utils::is_stable(&new_coefficients) {
                    success = false;
                } else {
                    *eq = *new_eq;
                    for channel_filters in self.filters.iter_mut() {
                        channel_filters[i].set_coefficients(new_coefficients, reset_state);
                    }
                }
            }
        }
        success
    }

    pub fn process(
        &mut self,
        params: sync::Arc<params::PluginParams>,
        buffer: &mut nih::Buffer,
    ) -> nih::ProcessStatus {
        for left_and_right in buffer.iter_samples() {
            self.update_filters(
                &params.eqs(),
                params.sample_rate.load(sync::atomic::Ordering::Relaxed),
                false,
            );
            for (filters, sample) in self.filters.iter_mut().zip(left_and_right) {
                *sample = biquad::utils::process_sequential(filters, *sample);
            }
        }

        nih::ProcessStatus::Normal
    }
}
