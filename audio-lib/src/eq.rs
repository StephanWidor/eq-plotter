use crate::utils;
use enum_table::Enumable;

#[derive(Debug, PartialEq, Clone, Copy, enum_table::Enumable)]
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
    Bypassed,
}

impl EqType {
    pub const ALL: &'static [EqType] = Enumable::VARIANTS;
    pub const VARIANT_COUNT: usize = Self::COUNT;

    pub const ALL_NAMES: [&'static str; Self::COUNT] = [
        "Volume",
        "Low Pass",
        "High Pass",
        "Band Pass",
        "AllPass",
        "Notch",
        "Peak",
        "Low Shelf",
        "High Shelf",
        "Bypassed",
    ];
    pub fn to_string(&self) -> &str {
        Self::ALL_NAMES[*self as usize]
    }

    pub const fn is_active(&self) -> bool {
        match self {
            EqType::Bypassed => false,
            _ => true,
        }
    }

    pub const fn has_frequency(&self) -> bool {
        match self {
            EqType::Volume => false,
            EqType::Bypassed => false,
            _ => true,
        }
    }

    pub const fn has_gain_db(&self) -> bool {
        match self {
            EqType::Volume => true,
            EqType::Peak => true,
            EqType::LowShelf => true,
            EqType::HighShelf => true,
            _ => false,
        }
    }

    pub const fn has_q(&self) -> bool {
        match self {
            EqType::Volume => false,
            EqType::Bypassed => false,
            _ => true,
        }
    }
}

impl TryFrom<usize> for EqType {
    type Error = &'static str;

    fn try_from(index: usize) -> Result<Self, Self::Error> {
        if index < Self::COUNT {
            Ok(Self::ALL[index])
        } else {
            Err(stringify!("EqType for index {} is not defined", index))
        }
    }
}

impl TryFrom<&str> for EqType {
    type Error = &'static str;

    fn try_from(type_name: &str) -> Result<Self, Self::Error> {
        let index_option = Self::ALL_NAMES.iter().position(|&name| name == type_name);
        match index_option {
            Some(index) => Ok(Self::ALL[index]),
            None => Err(stringify!("EqType {} is not defined", value)),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
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

#[derive(Debug, PartialEq, Clone, Copy)]
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Eq<F: utils::Float> {
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
    fn eq_type_from_and_into_string() {
        let round_trip_string = |name: &str| {
            let eq_type = EqType::try_from(name).unwrap();
            let name_back = eq_type.to_string();
            assert_eq!(name, name_back);
        };

        let round_trip_eq_type = |eq_type: EqType| {
            let name = eq_type.to_string();
            let eq_type_back = EqType::try_from(name).unwrap();
            assert_eq!(eq_type, eq_type_back);
        };

        for eq_type_name in EqType::ALL_NAMES {
            round_trip_string(eq_type_name);
        }

        for eq_type in EqType::ALL {
            round_trip_eq_type(*eq_type);
        }
    }

    #[test]
    fn eq_from_and_into() {
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
