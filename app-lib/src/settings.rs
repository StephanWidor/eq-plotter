use audio_lib::*;
use std::ops::RangeInclusive;

#[derive(Debug, Clone)]
pub struct Settings<F: utils::Float, const NUM_BANDS: usize> {
    pub eq_ranges: EqRanges<F>,
    pub impulse_response: ImpulseResponse<F>,
    pub init_eqs: [eq::Eq<F>; NUM_BANDS],
    pub init_sample_rate: F,
}

impl<F: utils::Float, const NUM_BANDS: usize> Default for Settings<F, NUM_BANDS> {
    fn default() -> Self {
        let eq_ranges = EqRanges::<F>::default();
        let log_frequency_step = (*eq_ranges.log_frequency_range.end()
            - *eq_ranges.log_frequency_range.start())
            / F::from(NUM_BANDS + 1).unwrap();
        let active_index = (F::from(NUM_BANDS).unwrap() / F::TWO).to_usize().unwrap();
        let eqs = std::array::from_fn(|i| {
            let frequency = eq::Frequency::LogHz(
                *eq_ranges.log_frequency_range.start()
                    + F::from(i + 1).unwrap() * log_frequency_step,
            );
            if i == active_index {
                eq::Eq {
                    gain: eq::Gain::Db(F::from(3).unwrap()),
                    frequency: frequency,
                    q: F::from(0.7).unwrap(),
                    eq_type: eq::EqType::Peak,
                }
            } else {
                eq::Eq {
                    gain: eq::Gain::Db(F::from(0).unwrap()),
                    frequency: frequency,
                    q: F::from(0.7).unwrap(),
                    eq_type: eq::EqType::Bypassed,
                }
            }
        });
        Self {
            eq_ranges,
            impulse_response: ImpulseResponse::default(),
            init_eqs: eqs,
            init_sample_rate: F::from(48000).unwrap(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EqRanges<F: utils::Float> {
    pub db_range: RangeInclusive<F>,
    pub log_frequency_range: RangeInclusive<F>,
    pub q_range: RangeInclusive<F>,
}

impl<F: utils::Float> EqRanges<F> {
    pub fn frequency_range(&self) -> RangeInclusive<F> {
        utils::log_to_frequency(*self.log_frequency_range.start())
            ..=utils::log_to_frequency(*self.log_frequency_range.end())
    }
}

impl<F: utils::Float> Default for EqRanges<F> {
    fn default() -> Self {
        let db_range = F::from(-40).unwrap()..=F::from(40).unwrap();
        let frequency_range = F::from(10).unwrap()..=F::from(20000).unwrap();
        let log_frequency_range = utils::frequency_to_log(*frequency_range.start())
            ..=utils::frequency_to_log(*frequency_range.end());
        Self {
            db_range: db_range,
            log_frequency_range: log_frequency_range,
            q_range: F::from(0.1).unwrap()..=F::from(10).unwrap(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImpulseResponse<F: utils::Float> {
    pub eps: F,
    pub hold_length: usize,
    pub max_length: usize,
}

impl<F: utils::Float> Default for ImpulseResponse<F> {
    fn default() -> Self {
        Self {
            eps: F::from(0.001).unwrap(),
            hold_length: 10,
            max_length: 1024,
        }
    }
}
