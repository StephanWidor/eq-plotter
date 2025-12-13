use num_traits::Float;
use variant_count::VariantCount;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Volume<F: Float> {
    pub gain_db: F,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct LowPass<F: Float> {
    pub cutoff_frequency: F,
    pub q: F,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct HighPass<F: Float> {
    pub cutoff_frequency: F,
    pub q: F,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct BandPass<F: Float> {
    pub frequency: F,
    pub q: F,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct AllPass<F: Float> {
    pub frequency: F,
    pub q: F,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Notch<F: Float> {
    pub frequency: F,
    pub q: F,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Peak<F: Float> {
    pub frequency: F,
    pub gain_db: F,
    pub q: F,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct LowShelf<F: Float> {
    pub cutoff_frequency: F,
    pub gain_db: F,
    pub q: F,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct HighShelf<F: Float> {
    pub cutoff_frequency: F,
    pub gain_db: F,
    pub q: F,
}

#[derive(Debug, PartialEq, Clone, Copy, VariantCount)]
pub enum EQ<F: Float> {
    Volume(crate::eq::Volume<F>),
    LowPass(crate::eq::LowPass<F>),
    HighPass(crate::eq::HighPass<F>),
    BandPass(crate::eq::BandPass<F>),
    AllPass(crate::eq::AllPass<F>),
    Notch(crate::eq::Notch<F>),
    Peak(crate::eq::Peak<F>),
    LowShelf(crate::eq::LowShelf<F>),
    HighShelf(crate::eq::HighShelf<F>),
}

impl<F: Float> EQ<F> {
    pub fn to_string(&self) -> &str {
        match self {
            EQ::Volume(_) => "Volume",
            EQ::LowPass(_) => "Low Pass",
            EQ::HighPass(_) => "High Pass",
            EQ::BandPass(_) => "Band Pass",
            EQ::AllPass(_) => "All Pass",
            EQ::Notch(_) => "Notch",
            EQ::Peak(_) => "Peak",
            EQ::LowShelf(_) => "Low Shelf",
            EQ::HighShelf(_) => "High Shelf",
        }
    }

    pub fn has_frequency(&self) -> bool {
        match self {
            EQ::Volume(_) => false,
            _ => true,
        }
    }

    pub fn has_gain_db(&self) -> bool {
        match self {
            EQ::Volume(_) => true,
            EQ::Peak(_) => true,
            EQ::LowShelf(_) => true,
            EQ::HighShelf(_) => true,
            _ => false,
        }
    }

    pub fn has_q(&self) -> bool {
        match self {
            EQ::Volume(_) => false,
            _ => true,
        }
    }

    pub fn set_parameters(&mut self, frequency: F, gain_db: F, q: F) {
        match self {
            EQ::Volume(volume) => {
                volume.gain_db = gain_db;
            }
            EQ::LowPass(lowpass) => {
                lowpass.cutoff_frequency = frequency;
                lowpass.q = q;
            }
            EQ::HighPass(highpass) => {
                highpass.cutoff_frequency = frequency;
                highpass.q = q;
            }
            EQ::BandPass(bandpass) => {
                bandpass.frequency = frequency;
                bandpass.q = q;
            }
            EQ::AllPass(allpass) => {
                allpass.frequency = frequency;
                allpass.q = q;
            }
            EQ::Notch(notch) => {
                notch.frequency = frequency;
                notch.q = q;
            }
            EQ::Peak(peak) => {
                peak.frequency = frequency;
                peak.gain_db = gain_db;
                peak.q = q;
            }
            EQ::LowShelf(lowshelf) => {
                lowshelf.cutoff_frequency = frequency;
                lowshelf.gain_db = gain_db;
                lowshelf.q = q;
            }
            EQ::HighShelf(highshelf) => {
                highshelf.cutoff_frequency = frequency;
                highshelf.gain_db = gain_db;
                highshelf.q = q;
            }
        };
    }

    pub fn all(frequency: F, gain_db: F, q: F) -> [Self; 9] {
        [
            Self::Volume(Volume { gain_db: gain_db }),
            Self::LowPass(LowPass {
                cutoff_frequency: frequency,
                q: q,
            }),
            Self::HighPass(HighPass {
                cutoff_frequency: frequency,
                q: q,
            }),
            Self::BandPass(BandPass {
                frequency: frequency,
                q: q,
            }),
            Self::AllPass(AllPass {
                frequency: frequency,
                q: q,
            }),
            Self::Notch(Notch {
                frequency: frequency,
                q: q,
            }),
            Self::Peak(Peak {
                frequency: frequency,
                gain_db: gain_db,
                q: q,
            }),
            Self::LowShelf(LowShelf {
                cutoff_frequency: frequency,
                gain_db: gain_db,
                q: q,
            }),
            Self::HighShelf(HighShelf {
                cutoff_frequency: frequency,
                gain_db: gain_db,
                q: q,
            }),
        ]
    }
}
