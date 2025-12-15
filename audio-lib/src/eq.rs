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

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Eq<F: Float> {
    pub gain_db: F,
    pub frequency: F,
    pub q: F,
    pub eq_type: EqType,
}

impl<F: Float> Eq<F> {
    pub fn gain_db(&self) -> Option<F> {
        if self.eq_type.has_gain_db() {
            Some(self.gain_db)
        } else {
            None
        }
    }

    pub fn frequency(&self) -> Option<F> {
        if self.eq_type.has_frequency() {
            Some(self.frequency)
        } else {
            None
        }
    }

    pub fn q(&self) -> Option<F> {
        if self.eq_type.has_q() {
            Some(self.q)
        } else {
            None
        }
    }
}
