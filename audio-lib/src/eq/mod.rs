pub mod eq_type;
pub mod frequency;
pub mod gain;
pub mod multiband_type;

use crate::utils;
pub use eq_type::*;
pub use frequency::*;
pub use gain::*;
pub use multiband_type::*;

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
#[serde(bound = "F: utils::Float")]
pub struct Eq<F: utils::Float> {
    pub gain: Gain<F>,
    pub frequency: Frequency<F>,
    pub q: F,
    pub eq_type: EqType,
    pub makeup_gain: Gain<F>,
}

// TODO: I bet this can be done better
impl From<Eq<f32>> for Eq<f64> {
    fn from(eq: Eq<f32>) -> Eq<f64> {
        Self {
            gain: eq.gain.into(),
            frequency: eq.frequency.into(),
            q: eq.q as f64,
            eq_type: eq.eq_type,
            makeup_gain: eq.makeup_gain.into(),
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
            makeup_gain: eq.makeup_gain.into(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
#[serde(bound = "F: utils::Float")]
pub struct MultibandEq<F: utils::Float, const NUM_BANDS: usize> {
    #[serde(with = "serde_arrays")]
    pub eqs: [Eq<F>; NUM_BANDS],
    pub processing_type: MultibandType,
}
