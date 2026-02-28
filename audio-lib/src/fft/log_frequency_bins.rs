use crate::{fft::*, utils::is_power_of_two, *};

#[derive(Debug, Clone)]
pub struct LogFrequencyBin<F: utils::Float> {
    pub index_range: std::ops::Range<usize>,
    pub frequency_range: std::ops::RangeInclusive<F>,
}

#[derive(Debug, Clone)]
pub struct LogFrequencyBins<F: utils::Float, const NUM_BINS: usize> {
    sample_rate: F,
    bins: [LogFrequencyBin<F>; NUM_BINS],
}

impl<F: utils::Float, const NUM_BINS: usize> LogFrequencyBins<F, NUM_BINS> {
    const _NUM_BINS_CHECK: () = assert!(NUM_BINS > 0);
    const FFT_LENGTH: usize = 1 << NUM_BINS;

    pub fn new(sample_rate: F) -> Self {
        let fft_length: usize = 1 << NUM_BINS;
        assert!(is_power_of_two(fft_length));
        let mut out = Self {
            sample_rate: sample_rate,
            bins: std::array::from_fn(|_| LogFrequencyBin {
                index_range: 0..0,
                frequency_range: F::ZERO..=F::ZERO,
            }),
        };
        out.set_sample_rate(sample_rate);
        out
    }

    pub fn set_sample_rate(&mut self, sample_rate: F) {
        let frequency_step = frequency_step(Self::FFT_LENGTH, sample_rate);
        let smallest_log_frequency = utils::frequency_to_log(frequency_step);
        let log_frequency_step = utils::frequency_to_log(F::TWO);
        let half_log_frequency_step = F::ONE_HALF * log_frequency_step;
        self.bins[0] = LogFrequencyBin {
            index_range: 1..2,
            frequency_range: (smallest_log_frequency - half_log_frequency_step)
                ..=(smallest_log_frequency + half_log_frequency_step),
        };
        for log_index in 1..NUM_BINS {
            let last_entry = &self.bins[log_index - 1];
            let start_index = last_entry.index_range.end;
            let start_log_frequency = *last_entry.frequency_range.end();
            let end_log_frequency = smallest_log_frequency
                + (F::from(log_index).unwrap() + F::ONE_HALF) * log_frequency_step;
            let end_frequency = utils::log_to_frequency(end_log_frequency);
            let end_index = (end_frequency / frequency_step).ceil().to_usize().unwrap();
            self.bins[log_index] = LogFrequencyBin {
                index_range: start_index..end_index,
                frequency_range: start_log_frequency..=end_log_frequency,
            };
        }
    }

    pub fn sample_rate(&self) -> F {
        self.sample_rate
    }

    pub fn bins(&self) -> &[LogFrequencyBin<F>; NUM_BINS] {
        &self.bins
    }
}
