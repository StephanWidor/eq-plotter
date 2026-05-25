use super::*;

pub struct ShowParams {
    pub gain: atomic::AtomicBool,
    pub signal_gain_spectrum: atomic::AtomicBool,
    pub phase: atomic::AtomicBool,
    pub impulse_response: atomic::AtomicBool,
    pub poles_and_zeros: atomic::AtomicBool,
}

impl ShowParams {
    pub fn from_options(show_options: &ShowOptions) -> Self {
        Self {
            gain: atomic::AtomicBool::new(show_options.gain),
            signal_gain_spectrum: atomic::AtomicBool::new(show_options.signal_gain_spectrum),
            phase: atomic::AtomicBool::new(show_options.phase),
            impulse_response: atomic::AtomicBool::new(show_options.impulse_response),
            poles_and_zeros: atomic::AtomicBool::new(show_options.poles_and_zeros),
        }
    }

    pub fn store_options(&self, options: &ShowOptions) {
        self.gain.store(options.gain, atomic::Ordering::Relaxed);
        self.signal_gain_spectrum
            .store(options.signal_gain_spectrum, atomic::Ordering::Relaxed);
        self.phase.store(options.phase, atomic::Ordering::Relaxed);
        self.impulse_response
            .store(options.impulse_response, atomic::Ordering::Relaxed);
        self.poles_and_zeros
            .store(options.poles_and_zeros, atomic::Ordering::Relaxed);
    }

    pub fn load_options(&self) -> ShowOptions {
        ShowOptions {
            gain: self.gain.load(atomic::Ordering::Relaxed),
            signal_gain_spectrum: self.signal_gain_spectrum.load(atomic::Ordering::Relaxed),
            phase: self.phase.load(atomic::Ordering::Relaxed),
            impulse_response: self.impulse_response.load(atomic::Ordering::Relaxed),
            poles_and_zeros: self.poles_and_zeros.load(atomic::Ordering::Relaxed),
        }
    }
}
