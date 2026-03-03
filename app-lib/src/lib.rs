use audio_lib::*;
use std::ops::RangeInclusive;

#[derive(Debug, Clone)]
pub struct Config<F: utils::Float> {
    db_range: RangeInclusive<F>,
    frequency_range: RangeInclusive<F>,
    log_frequency_range: RangeInclusive<F>,
    q_range: RangeInclusive<F>,
    init_eq: eq::Eq<F>,
}

impl<F: utils::Float> Config<F> {
    pub fn new(
        db_range: &RangeInclusive<F>,
        frequency_range: &RangeInclusive<F>,
        q_range: &RangeInclusive<F>,
        init_eq: &eq::Eq<F>,
    ) -> Self {
        Self {
            db_range: db_range.clone(),
            frequency_range: frequency_range.clone(),
            log_frequency_range: utils::frequency_to_log(*frequency_range.start())
                ..=utils::frequency_to_log(*frequency_range.end()),
            q_range: q_range.clone(),
            init_eq: *init_eq,
        }
    }

    pub fn db_range(&self) -> &RangeInclusive<F> {
        &self.db_range
    }

    pub fn frequency_range(&self) -> &RangeInclusive<F> {
        &self.frequency_range
    }

    pub fn log_frequency_range(&self) -> &RangeInclusive<F> {
        &self.log_frequency_range
    }

    pub fn q_range(&self) -> &RangeInclusive<F> {
        &self.q_range
    }

    pub fn init_eq(&self) -> &eq::Eq<F> {
        &self.init_eq
    }
}

impl<F: utils::Float> Default for Config<F> {
    fn default() -> Self {
        Config::new(
            &(F::from(-40).unwrap()..=F::from(40).unwrap()),
            &(F::from(10).unwrap()..=F::from(20000).unwrap()),
            &(F::from(0.1).unwrap()..=F::from(10).unwrap()),
            &eq::Eq {
                gain: eq::Gain::Db(F::from(3).unwrap()),
                frequency: eq::Frequency::Hz(F::from(440).unwrap()),
                q: F::from(0.7).unwrap(),
                eq_type: eq::EqType::Peak,
            },
        )
    }
}
