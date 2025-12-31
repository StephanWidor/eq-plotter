use crate::*;
use audio_lib::*;
use nih_plug::prelude as nih;
use std::sync;

pub struct Processor {
    eq: eq::Eq<f32>,
    filter: biquad::filter::Filter<f32>,
}

impl Default for Processor {
    fn default() -> Self {
        Self {
            eq: Self::INIT_EQ,
            filter: biquad::filter::Filter::new(&Self::INIT_FILTER_COEFFICIENTS),
        }
    }
}

impl Processor {
    const INIT_FILTER_COEFFICIENTS: biquad::coefficients::Coefficients<f32> =
        biquad::coefficients::Coefficients::from_volume_linear(0f32);
    const INIT_EQ: eq::Eq<f32> = eq::Eq {
        gain: eq::Gain::Db(std::f32::NEG_INFINITY),
        frequency: eq::Frequency::LogHz(std::f32::NEG_INFINITY),
        q: 0.0,
        eq_type: eq::EqType::Volume,
    };

    pub fn initialize(&mut self, params: sync::Arc<params::PluginParams>) -> bool {
        self.update(
            eq::Eq {
                gain: eq::Gain::Db(params.gain_db.value()),
                frequency: eq::Frequency::LogHz(params.log_frequency.value()),
                q: params.q.smoothed.next(),
                eq_type: params.eq_type.value().into(),
            },
            params.sample_rate.load(sync::atomic::Ordering::Relaxed),
            true,
        )
    }

    fn update(&mut self, eq: eq::Eq<f32>, sample_rate: f32, reset_state: bool) -> bool {
        if self.eq != eq {
            let new_coefficients = biquad::coefficients::Coefficients::from_eq(&eq, sample_rate);
            if !biquad::utils::is_stable(&new_coefficients) {
                return false;
            }
            self.eq = eq;
            self.filter.set_coefficients(new_coefficients, reset_state);
        }
        true
    }

    pub fn process(
        &mut self,
        params: sync::Arc<params::PluginParams>,
        buffer: &mut nih::Buffer,
    ) -> nih::ProcessStatus {
        assert!(buffer.channels() == 1); // we are always mono
        for channel_samples in buffer.iter_samples() {
            self.update(
                eq::Eq {
                    gain: eq::Gain::Db(params.gain_db.smoothed.next()),
                    frequency: eq::Frequency::LogHz(params.log_frequency.smoothed.next()),
                    q: params.q.smoothed.next(),
                    eq_type: params.eq_type.value().into(),
                },
                params.sample_rate.load(sync::atomic::Ordering::Relaxed),
                false,
            );

            for sample in channel_samples {
                *sample = self.filter.process(*sample);
            }
        }

        nih::ProcessStatus::Normal
    }
}
