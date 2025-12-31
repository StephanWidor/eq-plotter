use crate::utils;
use num_traits::Float;

#[derive(Debug, PartialEq, Clone, Copy, variant_count::VariantCount)]
pub enum EqType {
    Volume,
    LowPass,
    HighPass,
    BandPass,
    AllPass,
    Notch,
    Peak,
    LowShelf,
    HighShelf,
}

impl EqType {
    pub fn to_string(&self) -> &str {
        match self {
            EqType::Volume => "Volume",
            EqType::LowPass => "Low Pass",
            EqType::HighPass => "High Pass",
            EqType::BandPass => "Band Pass",
            EqType::AllPass => "All Pass",
            EqType::Notch => "Notch",
            EqType::Peak => "Peak",
            EqType::LowShelf => "Low Shelf",
            EqType::HighShelf => "High Shelf",
        }
    }

    pub fn has_frequency(&self) -> bool {
        match self {
            EqType::Volume => false,
            _ => true,
        }
    }

    pub fn has_gain_db(&self) -> bool {
        match self {
            EqType::Volume => true,
            EqType::Peak => true,
            EqType::LowShelf => true,
            EqType::HighShelf => true,
            _ => false,
        }
    }

    pub fn has_q(&self) -> bool {
        match self {
            EqType::Volume => false,
            _ => true,
        }
    }

    pub const ALL: [Self; Self::VARIANT_COUNT] = [
        Self::Volume,
        Self::LowPass,
        Self::HighPass,
        Self::BandPass,
        Self::AllPass,
        Self::Notch,
        Self::Peak,
        Self::LowShelf,
        Self::HighShelf,
    ];
}

impl TryFrom<&str> for EqType {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Volume" => Ok(EqType::Volume),
            "Low Pass" => Ok(EqType::LowPass),
            "High Pass" => Ok(EqType::HighPass),
            "Band Pass" => Ok(EqType::BandPass),
            "All Pass" => Ok(EqType::AllPass),
            "Notch" => Ok(EqType::Notch),
            "Peak" => Ok(EqType::Peak),
            "Low Shelf" => Ok(EqType::LowShelf),
            "High Shelf" => Ok(EqType::HighShelf),
            _ => Err(stringify!("EqType {} is not defined", value)),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Gain<F: Float> {
    Amplitude(F),
    Db(F),
}

impl<F: Float> Gain<F> {
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Frequency<F: Float> {
    Hz(F),
    LogHz(F),
}

impl<F: Float> Frequency<F> {
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Eq<F: Float> {
    pub gain: Gain<F>,
    pub frequency: Frequency<F>,
    pub q: F,
    pub eq_type: EqType,
}

// TODO: I bet this can be done better
impl From<Eq<f32>> for Eq<f64> {
    fn from(eq: Eq<f32>) -> Eq<f64> {
        Self {
            gain: eq.gain.into(),
            frequency: eq.frequency.into(),
            q: eq.q as f64,
            eq_type: eq.eq_type,
        }
    }
}
impl From<Eq<f64>> for Eq<f32> {
    fn from(eq: Eq<f64>) -> Eq<f32> {
        Self {
            gain: eq.gain.into(),
            frequency: eq.frequency.into(),
            q: eq.q as f32,
            eq_type: eq.eq_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_and_into() {
        let eq_f32 = Eq {
            gain: Gain::Db(-3.0_f32),
            frequency: Frequency::Hz(440.0_f32),
            q: 0.707_f32,
            eq_type: EqType::Peak,
        };
        let eq_f64: Eq<f64> = Eq::<f64>::from(eq_f32);

        let eq_f32_back: Eq<f32> = eq_f64.into();
        assert_eq!(eq_f32, eq_f32_back);
    }
}
