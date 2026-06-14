use super::*;

#[derive(PartialEq, Clone, Copy)]
pub struct Wrapper {
    multiband_type: eq::MultibandType,
}

impl From<eq::MultibandType> for Wrapper {
    fn from(multiband_type: eq::MultibandType) -> Self {
        Self {
            multiband_type: multiband_type,
        }
    }
}

impl Into<eq::MultibandType> for Wrapper {
    fn into(self) -> eq::MultibandType {
        self.multiband_type
    }
}

impl nice::Enum for Wrapper {
    fn variants() -> &'static [&'static str] {
        &eq::MultibandType::ALL_NAMES
    }

    fn ids() -> Option<&'static [&'static str]> {
        None
    }

    fn to_index(self) -> usize {
        self.multiband_type as usize
    }

    fn from_index(index: usize) -> Self {
        let from_result = eq::MultibandType::try_from(index);
        match from_result {
            Ok(multiband_type) => Self {
                multiband_type: multiband_type,
            },
            _ => Self {
                multiband_type: eq::MultibandType::try_from(0).unwrap(),
            },
        }
    }
}

pub type Param = nice::EnumParam<Wrapper>;
