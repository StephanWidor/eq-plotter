#[derive(Clone, Copy, enum_table::Enumable)]
pub enum ShowOptionType {
    Gain,
    Phase,
    ImpulseResponse,
    PolesAndZeros,
}

pub struct ShowOptions {
    pub gain: bool,
    pub signal_gain_spectrum: bool,
    pub phase: bool,
    pub impulse_response: bool,
    pub poles_and_zeros: bool,
}

impl ShowOptions {
    pub fn new_all_enabled() -> Self {
        Self {
            gain: true,
            signal_gain_spectrum: true,
            phase: true,
            impulse_response: true,
            poles_and_zeros: true,
        }
    }

    pub fn new_only_gain() -> Self {
        Self {
            gain: true,
            signal_gain_spectrum: true,
            phase: false,
            impulse_response: false,
            poles_and_zeros: false,
        }
    }
}
