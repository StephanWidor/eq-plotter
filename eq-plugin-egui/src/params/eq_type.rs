use super::*;

#[derive(PartialEq, Clone, Copy)]
pub struct Wrapper {
    eq_type: eq::EqType,
}

impl From<eq::EqType> for Wrapper {
    fn from(eq_type: eq::EqType) -> Self {
        Self { eq_type: eq_type }
    }
}

impl Into<eq::EqType> for Wrapper {
    fn into(self) -> eq::EqType {
        self.eq_type
    }
}

impl nice::Enum for Wrapper {
    fn variants() -> &'static [&'static str] {
        &eq::EqType::ALL_NAMES
    }

    fn ids() -> Option<&'static [&'static str]> {
        None
    }

    fn to_index(self) -> usize {
        self.eq_type as usize
    }

    fn from_index(index: usize) -> Self {
        let from_result = eq::EqType::try_from(index);
        match from_result {
            Ok(eq_type) => Self { eq_type: eq_type },
            _ => Self {
                eq_type: eq::EqType::try_from(0).unwrap(),
            },
        }
    }
}

pub type Param = nice::EnumParam<Wrapper>;
