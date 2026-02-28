use crate::{fft::*, windows::*, *};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProcessingResult {
    Collecting,
    NewOutputAvailable,
}

impl<F: utils::Float + rustfft::FftNum> Processor<F> {
    pub fn new(fft_length: usize, window_type: WindowType) -> Self {
        assert!(utils::is_power_of_two(fft_length));
        let mut planner = rustfft::FftPlanner::<F>::new();
        let fft = planner.plan_fft(fft_length, rustfft::FftDirection::Forward);
        let scratch_length = fft.get_inplace_scratch_len();

        Self {
            fft: fft,
            in_signal: Vec::with_capacity(fft_length),
            out_signal: vec![F::ZERO.into(); fft_length],
            scratch: vec![F::ZERO.into(); scratch_length],
            window: make_window(fft_length, window_type),
        }
    }

    pub fn reset(&mut self, fft_length: usize, window_type: WindowType) {
        assert!(utils::is_power_of_two(fft_length));
        let mut planner = rustfft::FftPlanner::<F>::new();
        self.fft = planner.plan_fft(fft_length, rustfft::FftDirection::Forward);
        self.in_signal.clear();
        self.in_signal.reserve(fft_length);
        self.out_signal.clear();
        self.out_signal.resize(fft_length, F::ZERO.into());
        self.scratch
            .resize(self.fft.get_inplace_scratch_len(), F::ZERO.into());
        self.window = make_window(fft_length, window_type);
    }

    pub fn push(&mut self, sample: F) -> ProcessingResult {
        self.in_signal.push(sample.into());
        if self.in_signal.len() == self.fft.len() {
            self.process_fft();
            ProcessingResult::NewOutputAvailable
        } else {
            ProcessingResult::Collecting
        }
    }

    pub fn append(&mut self, samples: &[F]) -> ProcessingResult {
        let append_samples = &mut |in_signal: &mut Vec<num::Complex<F>>, slice: &[F]| {
            in_signal.extend(
                slice
                    .iter()
                    .map(|&sample| num::Complex::new(sample, F::ZERO)),
            );
        };

        let samples_length = samples.len();
        let fft_length = self.fft.len();
        let remaining_capacity = fft_length - self.in_signal.len();

        if samples_length > fft_length {
            // not sure if this is good: we are omitting samples
            self.in_signal.clear();
            append_samples(
                &mut self.in_signal,
                &samples[(samples_length - fft_length)..samples_length],
            );
            self.process_fft();
            ProcessingResult::NewOutputAvailable
        } else if samples_length >= remaining_capacity {
            append_samples(&mut self.in_signal, &samples[0..remaining_capacity]);
            self.process_fft();
            if samples_length > remaining_capacity {
                append_samples(
                    &mut self.in_signal,
                    &samples[remaining_capacity..samples_length],
                );
            }
            ProcessingResult::NewOutputAvailable
        } else {
            append_samples(&mut self.in_signal, samples);
            ProcessingResult::Collecting
        }
    }

    pub fn out_signal(&self) -> &Vec<num::Complex<F>> {
        &self.out_signal
    }

    pub fn fft_length(&self) -> usize {
        self.fft.len()
    }

    pub fn frequency_step(&self, sample_rate: F) -> F {
        frequency_step(self.fft_length(), sample_rate)
    }

    fn process_fft(&mut self) {
        for i in 0..self.in_signal.len() {
            self.in_signal[i] = self.in_signal[i] * self.window[i];
        }
        self.fft.process_outofplace_with_scratch(
            &mut self.in_signal,
            &mut self.out_signal,
            &mut self.scratch,
        );
        self.scratch
            .resize(self.fft.get_inplace_scratch_len(), F::ZERO.into());
        self.in_signal.clear();
    }
}

pub struct Processor<F: utils::Float> {
    fft: std::sync::Arc<dyn rustfft::Fft<F>>,
    in_signal: Vec<num::Complex<F>>,
    out_signal: Vec<num::Complex<F>>,
    scratch: Vec<num::Complex<F>>,
    window: Vec<F>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;
    use num::complex::ComplexFloat;

    fn make_sine_wave(frequency: f64, sample_rate: f64, num_samples: usize) -> Vec<f64> {
        let mut signal = Vec::with_capacity(num_samples);
        for n in 0..num_samples {
            let sample = (2.0 * std::f64::consts::PI * frequency * n as f64 / sample_rate).cos();
            signal.push(sample);
        }
        signal
    }

    #[test]
    fn test_sine_waves() {
        let fft_length = 512;
        let nyquist_length = fft_length / 2;
        let mut processor: Processor<f64> = Processor::new(fft_length, WindowType::None);
        let sample_rate = 44100.0;
        let frequency_step = processor.frequency_step(sample_rate);
        let expect_load = num::Complex::new((fft_length / 2) as f64, 0.0);
        for i in 1..nyquist_length {
            let frequency = frequency_step * i as f64;
            let in_signal = make_sine_wave(frequency, sample_rate, fft_length);
            for sample in in_signal {
                processor.push(sample);
            }
            let out_signal = processor.out_signal();
            for j in 1..nyquist_length {
                let load = out_signal[j];
                if i == j {
                    assert_approx_eq!(load, expect_load);
                } else {
                    assert_approx_eq!(load, num::Complex::<f64>::ZERO);
                }
                assert_approx_eq!(out_signal[j], out_signal[fft_length - j]);
            }
        }
    }

    #[test]
    fn test_dirac() {
        let fft_length = 512;
        let mut processor: Processor<f64> = Processor::new(fft_length, WindowType::None);
        let mut in_signal = vec![0.0; fft_length];
        in_signal[0] = 1.0;
        processor.append(&in_signal);
        let out_signal = processor.out_signal();
        for s in out_signal {
            assert_approx_eq!(s, num::Complex::<f64>::ONE);
        }
    }

    #[test]
    fn test_direct_current() {
        let fft_length = 512;
        let mut processor: Processor<f64> = Processor::new(fft_length, WindowType::None);
        let in_signal = vec![1.0; fft_length];
        processor.append(&in_signal);
        let out_signal = processor.out_signal();
        assert_approx_eq!(out_signal[0], num::Complex::new(fft_length as f64, 0.0));
        for i in 1..fft_length {
            assert_approx_eq!(out_signal[i], num::Complex::<f64>::ZERO);
        }
    }

    #[test]
    fn test_appending_and_resetting() {
        let fft_length = 256;
        let mut processor: Processor<f64> = Processor::new(fft_length, WindowType::None);
        let sample_rate = 48000.0;
        let bin_index = 10;
        let frequency = (bin_index as f64) * processor.frequency_step(sample_rate);
        let signal_length = 3 * fft_length;
        let split_length_0 = 200;
        let split_length_1 = 300;
        let signal = make_sine_wave(frequency, sample_rate, signal_length);

        let mut processing_result = processor.append(&signal[0..split_length_0]);
        assert_eq!(processing_result, ProcessingResult::Collecting);
        for out in processor.out_signal() {
            assert_eq!(*out, num::Complex::new(0.0, 0.0));
        }

        let expect_load = (fft_length / 2) as f64;

        processing_result = processor.append(&signal[split_length_0..split_length_1]);
        assert_eq!(processing_result, ProcessingResult::NewOutputAvailable);
        assert_approx_eq!(processor.out_signal()[bin_index].re, expect_load);

        processor.reset(fft_length / 2, WindowType::Hamming);
        for out in processor.out_signal() {
            assert_eq!(*out, num::Complex::new(0.0, 0.0));
        }

        processing_result = processor.append(&signal);
        assert_eq!(processing_result, ProcessingResult::NewOutputAvailable);
        let max_load = processor
            .out_signal()
            .iter()
            .map(|s| s.re)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        assert_eq!(max_load, processor.out_signal()[bin_index / 2].re);
    }
}
