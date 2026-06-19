use audio_lib::*;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(bound = "F: utils::Float")]
pub enum Selection<F: utils::Float, const NUM_BANDS: usize> {
    None,
    Selected(String, eq::MultibandEq<F, NUM_BANDS>),
    SelectedAndChanged(String, eq::MultibandEq<F, NUM_BANDS>),
}

impl<F: utils::Float, const NUM_BANDS: usize> Selection<F, NUM_BANDS> {
    pub fn mark_as_changed(&mut self) {
        if let Selection::Selected(name, preset) = &self {
            *self = Selection::SelectedAndChanged(name.clone(), preset.clone()); // TODO: can we do without cloning?
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(bound = "F: utils::Float")]
pub struct Presets<F: utils::Float, const NUM_BANDS: usize> {
    preset_map: HashMap<String, eq::MultibandEq<F, NUM_BANDS>>,
}

impl<F: utils::Float, const NUM_BANDS: usize> Presets<F, NUM_BANDS> {
    pub fn new() -> Self {
        Self {
            preset_map: HashMap::new(),
        }
    }

    pub fn new_with_init(init_name: String, init_eq: eq::MultibandEq<F, NUM_BANDS>) -> Self {
        Self {
            preset_map: HashMap::from([(init_name, init_eq)]),
        }
    }

    pub fn add(&mut self, name: String, eq: eq::MultibandEq<F, NUM_BANDS>) -> bool {
        if self.preset_map.contains_key(&name) {
            return false;
        }
        self.preset_map.insert(name, eq);
        true
    }

    pub fn force_add(&mut self, name: String, eq: eq::MultibandEq<F, NUM_BANDS>) {
        if let Some(preset) = self.preset_map.get_mut(&name) {
            *preset = eq;
        } else {
            self.preset_map.insert(name, eq);
        }
    }

    pub fn get(&self, name: &String) -> Option<&eq::MultibandEq<F, NUM_BANDS>> {
        if let Some(eq) = self.preset_map.get(name) {
            Some(&eq)
        } else {
            None
        }
    }

    pub fn remove(&mut self, name: &String) {
        self.preset_map.remove(name);
    }

    pub fn count(&self) -> usize {
        self.preset_map.iter().count()
    }

    pub fn contains(&self, preset_name: &str) -> bool {
        self.preset_map.contains_key(preset_name)
    }

    pub fn names_iter(&self) -> impl Iterator<Item = &String> {
        self.preset_map.iter().map(|p| p.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct TestSetup {
        pub eqs: Vec<(String, eq::MultibandEq<f32, 2>)>,
    }

    impl TestSetup {
        fn new() -> Self {
            Self {
                eqs: vec![
                    (
                        String::from("a preset"),
                        eq::MultibandEq {
                            eqs: [
                                eq::Eq {
                                    gain: eq::Gain::Db(0.0),
                                    frequency: eq::Frequency::Hz(1000.0),
                                    q: 0.7,
                                    eq_type: eq::EqType::Peak,
                                    makeup_gain: eq::Gain::Amplitude(0.8),
                                },
                                eq::Eq {
                                    gain: eq::Gain::Db(-3.0),
                                    frequency: eq::Frequency::Hz(2000.0),
                                    q: 1.4,
                                    eq_type: eq::EqType::LowShelf,
                                    makeup_gain: eq::Gain::Amplitude(1.3),
                                },
                            ],
                            processing_type: eq::MultibandType::Parallel,
                        },
                    ),
                    (
                        String::from("another preset"),
                        eq::MultibandEq {
                            eqs: [
                                eq::Eq {
                                    gain: eq::Gain::Db(6.0),
                                    frequency: eq::Frequency::Hz(4000.0),
                                    q: 0.5,
                                    eq_type: eq::EqType::HighPass,
                                    makeup_gain: eq::Gain::Db(1.3),
                                },
                                eq::Eq {
                                    gain: eq::Gain::Db(3.0),
                                    frequency: eq::Frequency::Hz(1000.0),
                                    q: 1.0,
                                    eq_type: eq::EqType::Peak,
                                    makeup_gain: eq::Gain::Amplitude(1.3),
                                },
                            ],
                            processing_type: eq::MultibandType::Sequential,
                        },
                    ),
                    (
                        String::from("still another preset"),
                        eq::MultibandEq {
                            eqs: [
                                eq::Eq {
                                    gain: eq::Gain::Db(0.0),
                                    frequency: eq::Frequency::Hz(2000.0),
                                    q: 2.0,
                                    eq_type: eq::EqType::Notch,
                                    makeup_gain: eq::Gain::Db(-6.0),
                                },
                                eq::Eq {
                                    gain: eq::Gain::Db(-12.0),
                                    frequency: eq::Frequency::Hz(4000.0),
                                    q: 0.3,
                                    eq_type: eq::EqType::HighShelf,
                                    makeup_gain: eq::Gain::Amplitude(1.003),
                                },
                            ],
                            processing_type: eq::MultibandType::Parallel,
                        },
                    ),
                ],
            }
        }
    }

    #[test]
    fn test_adding_getting_and_removing() {
        let setup = TestSetup::new();
        let mut presets = Presets::new();
        let num_presets = setup.eqs.iter().count();

        for (index, (name, eqs)) in setup.eqs.iter().enumerate() {
            assert!(presets.add(name.clone(), eqs.clone()));
            assert_eq!(presets.count(), index + 1);
        }

        for (name, eqs) in setup.eqs.iter() {
            let eqs_back = presets.get(name);
            assert!(eqs_back.is_some());
            assert_eq!(eqs, eqs_back.unwrap());
        }

        for index in 0..setup.eqs.iter().count() {
            let next_index = (index + 1) % num_presets;
            let name = &setup.eqs[index].0;
            let next_eqs = &setup.eqs[next_index].1;
            assert!(!presets.add(name.clone(), next_eqs.clone()));

            presets.force_add(name.clone(), next_eqs.clone());
            let next_eqs_back = presets.get(name);
            assert!(next_eqs_back.is_some());
            assert_eq!(next_eqs, next_eqs_back.unwrap());
        }

        for (index, (name, _)) in setup.eqs.iter().enumerate() {
            presets.remove(name);
            assert_eq!(presets.count(), num_presets - index - 1);
            assert!(presets.get(name).is_none());
        }
    }
}
