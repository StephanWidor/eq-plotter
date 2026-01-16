use crate::*;
use audio_lib::*;
use nih_plug::prelude as nih;
use std::sync;

pub struct Processor {
    eqs: [eq::Eq<f32>; params::PluginParams::NUM_BANDS],
    filters: [biquad::filter::Filter<f32>; params::PluginParams::NUM_BANDS],
}

impl Default for Processor {
    fn default() -> Self {
        Self {
            eqs: [Self::INIT_EQ; params::PluginParams::NUM_BANDS],
            filters: array_init::array_init(|_| {
                biquad::filter::Filter::new(&Self::INIT_FILTER_COEFFICIENTS)
            }),
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

    pub fn initialize(&mut self, params: sync::Arc<params::PluginParams>) -> bool {
        self.update_filters(
            &params.eqs(),
            params.sample_rate.load(sync::atomic::Ordering::Relaxed),
            true,
        )
    }

    fn update_filters(&mut self, eqs: &[eq::Eq<f32>], sample_rate: f32, reset_state: bool) -> bool {
        let mut success = true;
        for ((new_eq, eq), filter) in eqs
            .iter()
            .zip(self.eqs.as_mut_slice())
            .zip(self.filters.as_mut_slice())
        {
            if *new_eq != *eq {
                let new_coefficients =
                    biquad::coefficients::Coefficients::from_eq(new_eq, sample_rate);
                if !biquad::utils::is_stable(&new_coefficients) {
                    success = false;
                }
                *eq = *new_eq;
                filter.set_coefficients(new_coefficients, reset_state);
            }
        }

        success
    }

    pub fn process(
        &mut self,
        params: sync::Arc<params::PluginParams>,
        buffer: &mut nih::Buffer,
    ) -> nih::ProcessStatus {
        assert!(buffer.channels() == 1); // we are always mono
        for channel_samples in buffer.iter_samples() {
            self.update_filters(
                &params.eqs(),
                params.sample_rate.load(sync::atomic::Ordering::Relaxed),
                false,
            );

            for sample in channel_samples {
                *sample = biquad::utils::process_sequential(&mut self.filters, *sample);
            }
        }

        nih::ProcessStatus::Normal
    }
}
