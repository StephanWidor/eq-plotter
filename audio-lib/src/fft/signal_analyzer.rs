use rustfft::FftNum;

use crate::*;
use std::sync;

#[derive(Debug, Clone)]
pub struct Coefficients<F: utils::Float> {
    pub sample_rate: F,
    pub window_type: windows::WindowType,
    pub attack_time: F,
    pub release_time: F,
}

pub struct SharedData<const NUM_BINS: usize, const NUM_CHANNELS: usize> {
    pub frequency_bins: sync::Arc<sync::RwLock<fft::LogFrequencyBins<f32, NUM_BINS>>>,
    pub linear_gains: spsc::swap::Swap<[[f32; NUM_BINS]; NUM_CHANNELS]>,
}

impl<const NUM_BINS: usize, const NUM_CHANNELS: usize> SharedData<NUM_BINS, NUM_CHANNELS> {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            frequency_bins: sync::Arc::new(sync::RwLock::new(fft::LogFrequencyBins::new(
                sample_rate,
            ))),
            linear_gains: spsc::swap::Swap::from_init_value(&[[0_f32; NUM_BINS]; NUM_CHANNELS]),
        }
    }

    pub fn reset(&self, sample_rate: f32) {
        self.frequency_bins
            .write()
            .unwrap()
            .set_sample_rate(sample_rate);
        self.linear_gains.producer.manipulate_and_push(&|gains| {
            for channel_gains in gains.iter_mut() {
                channel_gains.fill(0_f32);
            }
        });
    }
}

pub struct SignalAnalyzer<F: utils::Float, const NUM_BINS: usize, const NUM_CHANNELS: usize> {
    fft_processors: [fft::Processor<F>; NUM_CHANNELS],
    gain_analyzers: [GainProcessor<F, NUM_BINS>; NUM_CHANNELS],
}

impl<F: utils::Float + FftNum, const NUM_BINS: usize, const NUM_CHANNELS: usize>
    SignalAnalyzer<F, NUM_BINS, NUM_CHANNELS>
{
    pub fn new(coefficients: &Coefficients<F>) -> Self {
        let fft_length = 1 << NUM_BINS;
        Self {
            fft_processors: std::array::from_fn(|_| {
                fft::Processor::new(fft_length, coefficients.window_type)
            }),
            gain_analyzers: std::array::from_fn(|_| GainProcessor::new(coefficients)),
        }
    }

    pub fn reset(&mut self, coefficients: &Coefficients<F>) {
        let fft_length = 1 << NUM_BINS;
        for i in 0..NUM_CHANNELS {
            self.fft_processors[i].reset(fft_length, coefficients.window_type);
            self.gain_analyzers[i].reset(coefficients);
        }
    }

    pub fn push<T: AsRef<[F]>>(
        &mut self,
        buffer: &[T],
        frequency_bins: &fft::LogFrequencyBins<F, NUM_BINS>,
        shared_linear_gains: &spsc::swap::Producer<[[F; NUM_BINS]; NUM_CHANNELS]>,
    ) {
        assert!(buffer.len() <= NUM_CHANNELS);
        let mut needs_push = false;
        for channel in 0..buffer.len() {
            let channel_samples = buffer[channel].as_ref();
            let fft_result = self.fft_processors[channel].append(channel_samples);
            if fft_result == fft::ProcessingResult::NewOutputAvailable {
                let spectrum = self.fft_processors[channel].out_signal();
                self.gain_analyzers[channel].push(spectrum, frequency_bins);
                needs_push = true;
            }
        }
        if needs_push {
            shared_linear_gains.manipulate_and_push(&|push_data| {
                for channel in 0..NUM_CHANNELS {
                    let channel_gains = &mut push_data[channel];
                    for (i, gain) in self.gain_analyzers[channel].linear_gains().enumerate() {
                        channel_gains[i] = gain;
                    }
                }
            });
        }
    }
}

struct GainProcessor<F: utils::Float, const NUM_BINS: usize> {
    envelopes: [envelope_follower::EnvelopeFollower<F>; NUM_BINS],
    amplitude_square_scale: F,
}

impl<F: utils::Float, const NUM_BINS: usize> GainProcessor<F, NUM_BINS> {
    const NUM_BINS_CHECK: () = assert!(NUM_BINS > 0);
    const FFT_LENGTH: usize = 1 << NUM_BINS;

    fn new(coefficients: &Coefficients<F>) -> Self {
        let envelope_coefficients = Self::make_envelope_coefficients(coefficients);
        Self {
            envelopes: std::array::from_fn(|_| {
                envelope_follower::EnvelopeFollower::from_coefficients(&envelope_coefficients)
            }),
            amplitude_square_scale: Self::make_amplitude_square_scale(coefficients.window_type),
        }
    }

    fn reset(&mut self, coefficients: &Coefficients<F>) {
        let envelope_coefficients = Self::make_envelope_coefficients(coefficients);
        for envelope in self.envelopes.iter_mut() {
            envelope.set_coefficients(&envelope_coefficients);
            envelope.reset(F::ZERO);
        }
        self.amplitude_square_scale = Self::make_amplitude_square_scale(coefficients.window_type);
    }

    fn push(
        &mut self,
        fft_output: &[num::Complex<F>],
        frequency_bins: &fft::LogFrequencyBins<F, NUM_BINS>,
    ) {
        assert!(fft_output.len() == Self::FFT_LENGTH);
        for (i, bin) in frequency_bins.bins().iter().enumerate() {
            let mut accumulator = F::ZERO;
            for j in bin.index_range.clone() {
                accumulator += fft_output[j].norm_sqr();
            }
            accumulator *= self.amplitude_square_scale;
            self.envelopes[i].process(accumulator.sqrt());
        }
    }

    fn linear_gains(&self) -> impl Iterator<Item = F> {
        self.envelopes.iter().map(|envelope| envelope.value())
    }

    fn make_envelope_coefficients(
        coefficients: &Coefficients<F>,
    ) -> envelope_follower::Coefficients<F> {
        let time_scale = F::ONE / F::from(Self::FFT_LENGTH).unwrap();
        envelope_follower::Coefficients::new(
            coefficients.attack_time * time_scale,
            coefficients.release_time * time_scale,
            coefficients.sample_rate,
        )
    }

    fn make_amplitude_square_scale(window_type: windows::WindowType) -> F {
        let scale =
            F::TWO / (windows::center_value::<F>(window_type) * F::from(1 << NUM_BINS).unwrap());
        scale * scale
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use more_asserts::assert_ge;
    use more_asserts::assert_le;

    fn make_sine_wave(frequency: f32, sample_rate: f32, num_samples: usize) -> Vec<f32> {
        let mut signal = Vec::with_capacity(num_samples);
        for n in 0..num_samples {
            let sample = (2.0 * std::f32::consts::PI * frequency * n as f32 / sample_rate).cos();
            signal.push(sample);
        }
        signal
    }

    #[test]
    fn test_sine_waves() {
        const NUM_CHANNELS: usize = 1;
        const NUM_BINS: usize = 12;
        const COEFFICIENTS: Coefficients<f32> = Coefficients {
            sample_rate: 48000.0,
            attack_time: 0.01,
            release_time: 0.2,
            window_type: windows::WindowType::Hamming,
        };

        let fft_length = 1 << NUM_BINS;
        let shared_data = SharedData::<NUM_BINS, NUM_CHANNELS>::new(COEFFICIENTS.sample_rate);
        let mut analyzer = SignalAnalyzer::<f32, NUM_BINS, NUM_CHANNELS>::new(&COEFFICIENTS);

        let frequency_step = fft::frequency_step(fft_length, COEFFICIENTS.sample_rate);
        for bin_index in 0..NUM_BINS {
            let frequency = frequency_step * (1 << bin_index) as f32;
            let in_signal = [make_sine_wave(frequency, COEFFICIENTS.sample_rate, fft_length); 1];
            let bins = shared_data.frequency_bins.read().unwrap();
            for _i in 0..10 {
                analyzer.push(&in_signal, &bins, &shared_data.linear_gains.producer);
            }

            let gains_linear = shared_data.linear_gains.consumer.pull_and_read()[0];
            for i in 0..gains_linear.len() {
                let gain_linear = gains_linear[i];
                if i == bin_index {
                    assert_ge!(gain_linear, 0.9_f32);
                } else if (i as i32 - bin_index as i32).abs() <= 1 {
                    assert_le!(gain_linear, 0.5_f32);
                } else {
                    assert_le!(gain_linear, 0.1_f32);
                }
            }
        }
    }
}
