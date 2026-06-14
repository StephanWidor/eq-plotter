use crate::utils;

#[derive(Debug, PartialEq, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(bound = "F: utils::Float")]
pub enum Frequency<F: utils::Float> {
    Hz(F),
    LogHz(F),
}

impl<F: utils::Float> Frequency<F> {
    pub fn hz(&self) -> F {
        match self {
            Frequency::Hz(hz) => *hz,
            Frequency::LogHz(log_hz) => utils::log_to_frequency(*log_hz),
        }
    }
    pub fn log_hz(&self) -> F {
        match self {
            Frequency::Hz(hz) => utils::frequency_to_log(*hz),
            Frequency::LogHz(log_hz) => *log_hz,
        }
    }
}

// TODO: I bet this can be done better
impl From<Frequency<f32>> for Frequency<f64> {
    fn from(frequency: Frequency<f32>) -> Self {
        match frequency {
            Frequency::<f32>::Hz(hz) => Self::Hz(hz as f64),
            Frequency::<f32>::LogHz(log_hz) => Self::LogHz(log_hz as f64),
        }
    }
}
impl From<Frequency<f64>> for Frequency<f32> {
    fn from(frequency: Frequency<f64>) -> Self {
        match frequency {
            Frequency::<f64>::Hz(hz) => Self::Hz(hz as f32),
            Frequency::<f64>::LogHz(log_hz) => Self::LogHz(log_hz as f32),
        }
    }
}
