use audio_lib::eq;

pub const UI_BACKGROUND_COLOR: [u8; 3] = [32, 35, 38]; // [r,g,b]
pub const MIN_GAIN_DB: f64 = -20.0;
pub const MAX_GAIN_DB: f64 = 20.0;
pub const MIN_FREQUENCY: f64 = 10.0;
pub const MIN_LOG_FREQUENCY: f64 = 1.0; // 10.0.log10();
pub const MAX_FREQUENCY: f64 = 20000.0;
pub const MAX_LOG_FREQUENCY: f64 = 4.3010299956639813; // 20000.0.log10();
pub const MIN_Q: f64 = 0.1;
pub const MAX_Q: f64 = 10.0;
pub const DEFAULT_EQ: eq::Eq<f64> = eq::Eq {
    gain: eq::Gain::Db(-3.0),
    frequency: eq::Frequency::Hz(1000.0),
    q: 0.7,
    eq_type: eq::EqType::Peak,
};
