use audio_lib::*;

type PerChannelFilters<const NUM_BANDS: usize> = [biquad::filter::Filter<f32>; NUM_BANDS];

pub struct Processor<const NUM_CHANNELS: usize, const NUM_BANDS: usize> {
    eqs: [eq::Eq<f32>; NUM_BANDS],
    filters: [PerChannelFilters<NUM_BANDS>; NUM_CHANNELS],
}

impl<const NUM_CHANNELS: usize, const NUM_BANDS: usize> Default
    for Processor<NUM_CHANNELS, NUM_BANDS>
{
    fn default() -> Self {
        Self {
            eqs: [Self::INIT_EQ; NUM_BANDS],
            filters: std::array::from_fn(|_| Self::init_channel_filters()),
        }
    }
}

impl<const NUM_CHANNELS: usize, const NUM_BANDS: usize> Processor<NUM_CHANNELS, NUM_BANDS> {
    const INIT_FILTER_COEFFICIENTS: biquad::coefficients::Coefficients<f32> =
        biquad::coefficients::Coefficients::muted();
    const INIT_EQ: eq::Eq<f32> = eq::Eq {
        gain: eq::Gain::Db(std::f32::NEG_INFINITY),
        frequency: eq::Frequency::LogHz(std::f32::NEG_INFINITY),
        q: 0.0,
        eq_type: eq::EqType::Volume,
    };

    fn init_channel_filters() -> PerChannelFilters<NUM_BANDS> {
        std::array::from_fn(|_| biquad::filter::Filter::new(&Self::INIT_FILTER_COEFFICIENTS))
    }

    pub fn initialize(&mut self, eqs: &[eq::Eq<f32>], sample_rate: f32) -> bool {
        self.update_filters(eqs, sample_rate, true)
    }

    fn update_filters(
        &mut self,
        new_eqs: &[eq::Eq<f32>],
        sample_rate: f32,
        reset_state: bool,
    ) -> bool {
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
                    *eq = *new_eq;
                    for channel_filters in self.filters.iter_mut() {
                        channel_filters[i].set_coefficients(new_coefficients, reset_state);
                    }
                }
            }
        }
        success
    }

    pub fn process(&mut self, eqs: &[eq::Eq<f32>], sample_rate: f32, buffer: &mut [&'_ mut [f32]]) {
        self.update_filters(eqs, sample_rate, false);

        assert!(buffer.len() <= NUM_CHANNELS);
        for channel in 0..buffer.len() {
            let channel_samples = buffer.get_mut(channel).unwrap();
            let channel_filters = &mut self.filters[channel];
            for sample in (*channel_samples).iter_mut() {
                *sample = biquad::utils::process_sequential(channel_filters, *sample);
            }
        }
    }
}
