use crate::utils;

#[derive(Debug, PartialEq, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(bound = "F: utils::Float")]
pub enum Gain<F: utils::Float> {
    Amplitude(F),
    Db(F),
}

impl<F: utils::Float> Gain<F> {
    pub fn amplitude(&self) -> F {
        match self {
            Gain::Amplitude(amplitude) => *amplitude,
            Gain::Db(db) => utils::db_to_amplitude(*db),
        }
    }
    pub fn db(&self) -> F {
        match self {
            Gain::Amplitude(amplitude) => utils::amplitude_to_db(*amplitude),
            Gain::Db(db) => *db,
        }
    }
}

// TODO: I bet this can be done better
impl From<Gain<f32>> for Gain<f64> {
    fn from(gain: Gain<f32>) -> Self {
        match gain {
            Gain::<f32>::Amplitude(amplitude) => Self::Amplitude(amplitude as f64),
            Gain::<f32>::Db(db) => Self::Db(db as f64),
        }
    }
}
impl From<Gain<f64>> for Gain<f32> {
    fn from(gain: Gain<f64>) -> Self {
        match gain {
            Gain::<f64>::Amplitude(amplitude) => Self::Amplitude(amplitude as f32),
            Gain::<f64>::Db(db) => Self::Db(db as f32),
        }
    }
}
