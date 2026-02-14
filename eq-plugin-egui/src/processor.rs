use audio_lib::*;

type FilterCoefficients<const NUM_BANDS: usize> =
    [biquad::coefficients::Coefficients<f32>; NUM_BANDS];
type Filters<const NUM_BANDS: usize> = [biquad::filter::State<f32>; NUM_BANDS];

pub struct Processor<const NUM_CHANNELS: usize, const NUM_BANDS: usize> {
    eqs: [eq::Eq<f32>; NUM_BANDS],
    coefficients: FilterCoefficients<NUM_BANDS>,
    filters: [Filters<NUM_BANDS>; NUM_CHANNELS],
}

impl<const NUM_CHANNELS: usize, const NUM_BANDS: usize> Default
    for Processor<NUM_CHANNELS, NUM_BANDS>
{
    fn default() -> Self {
        Self {
            eqs: [Self::INIT_EQ; NUM_BANDS],
            coefficients: [Self::INIT_FILTER_COEFFICIENTS; NUM_BANDS],
            filters: std::array::from_fn(|_| std::array::from_fn(|_| biquad::filter::State::new())),
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

    pub fn initialize(&mut self, eqs: &[eq::Eq<f32>], sample_rate: f32) -> bool {
        for channel_filters in self.filters.iter_mut() {
            for filter in channel_filters.iter_mut() {
                filter.reset();
            }
        }
        self.update_coefficients(eqs, sample_rate)
    }

    pub fn process(&mut self, eqs: &[eq::Eq<f32>], sample_rate: f32, buffer: &mut [&'_ mut [f32]]) {
        self.update_coefficients(eqs, sample_rate);

        assert!(buffer.len() <= NUM_CHANNELS);
        for channel in 0..buffer.len() {
            let channel_samples = buffer.get_mut(channel).unwrap();
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
                    *eq = *new_eq;
                    self.coefficients[i] = new_coefficients;
                }
            }
        }
        success
    }
}
