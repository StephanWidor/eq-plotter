use crate::utils;

pub struct Delay<const CAPACITY: usize, F: utils::Float> {
    buffer: [F; CAPACITY],
    index: usize,
    delay_in_samples: usize,
}

impl<const CAPACITY: usize, F: utils::Float> Delay<CAPACITY, F> {
    const _CAPACITY_CHECK: () = assert!(CAPACITY > 0);

    pub fn new(delay_in_samples: usize) -> Self {
        assert!(delay_in_samples < CAPACITY);
        Self {
            buffer: [F::ZERO; CAPACITY],
            index: 0,
            delay_in_samples: delay_in_samples,
        }
    }

    pub fn set_delay(&mut self, delay_in_samples: usize) {
        assert!(delay_in_samples < CAPACITY);
        self.delay_in_samples = delay_in_samples;
    }

    pub fn get_delay_in_samples(&self) -> usize {
        self.delay_in_samples
    }

    pub fn process(&mut self, sample: F) -> F {
        self.index = (self.index + 1) % CAPACITY;
        self.buffer[self.index] = sample;
        let delayed_index = (self.index + CAPACITY - self.delay_in_samples) % CAPACITY;
        self.buffer[delayed_index]
    }

    pub fn reset(&mut self) {
        self.index = 0;
        self.buffer.fill(F::ZERO);
    }
}
