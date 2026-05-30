use audio_lib::*;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Selection {
    None,
    Selected(String),
    SelectedChanged(String),
}

impl Selection {
    pub fn mark_as_changed(&mut self) {
        if let Selection::Selected(selected_preset_name) = &self {
            *self = Selection::SelectedChanged(selected_preset_name.clone()); // TODO: can we do without cloning?
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(bound = "F: utils::Float")]
pub struct Presets<F: utils::Float, const NUM_BANDS: usize> {
    preset_map: HashMap<String, Preset<F, NUM_BANDS>>,
}

impl<F: utils::Float, const NUM_BANDS: usize> Presets<F, NUM_BANDS> {
    pub fn new() -> Self {
        Self {
            preset_map: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: String, eqs: [eq::Eq<F>; NUM_BANDS]) -> bool {
        if self.preset_map.contains_key(&name) {
            return false;
        }
        self.preset_map.insert(name, Preset { eqs });
        true
    }

    pub fn force_add(&mut self, name: String, eqs: [eq::Eq<F>; NUM_BANDS]) {
        if let Some(preset) = self.preset_map.get_mut(&name) {
            preset.set_eqs(&eqs);
        } else {
            self.preset_map.insert(name, Preset { eqs });
        }
    }

    pub fn get(&self, name: &String) -> Option<&[eq::Eq<F>; NUM_BANDS]> {
        if let Some(preset) = self.preset_map.get(name) {
            Some(&preset.eqs)
        } else {
            None
        }
    }

    pub fn get_inline(&self, name: &String, eqs: &mut [eq::Eq<F>; NUM_BANDS]) -> bool {
        if let Some(preset) = self.preset_map.get(name) {
            preset.get_eqs(eqs);
            true
        } else {
            false
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(bound = "F: utils::Float")]
pub struct Preset<F: utils::Float, const NUM_BANDS: usize> {
    #[serde(with = "serde_arrays")]
    pub eqs: [eq::Eq<F>; NUM_BANDS],
}

impl<F: utils::Float, const NUM_BANDS: usize> Preset<F, NUM_BANDS> {
    pub fn set_eqs(&mut self, eqs: &[eq::Eq<F>; NUM_BANDS]) {
        for i in 0..NUM_BANDS {
            self.eqs[i] = eqs[i].clone();
        }
    }

    pub fn get_eqs(&self, eqs: &mut [eq::Eq<F>; NUM_BANDS]) {
        for i in 0..NUM_BANDS {
            eqs[i] = self.eqs[i].clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct TestSetup {
        pub eqs: Vec<(String, [eq::Eq<f32>; 2])>,
    }

    impl TestSetup {
        fn new() -> Self {
            Self {
                eqs: vec![
                    (
                        String::from("a preset"),
                        [
                            eq::Eq {
                                gain: eq::Gain::Db(0.0),
                                frequency: eq::Frequency::Hz(1000.0),
                                q: 0.7,
                                eq_type: eq::EqType::Peak,
                            },
                            eq::Eq {
                                gain: eq::Gain::Db(-3.0),
                                frequency: eq::Frequency::Hz(2000.0),
                                q: 1.4,
                                eq_type: eq::EqType::LowShelf,
                            },
                        ],
                    ),
                    (
                        String::from("another preset"),
                        [
                            eq::Eq {
                                gain: eq::Gain::Db(6.0),
                                frequency: eq::Frequency::Hz(4000.0),
                                q: 0.5,
                                eq_type: eq::EqType::HighPass,
                            },
                            eq::Eq {
                                gain: eq::Gain::Db(3.0),
                                frequency: eq::Frequency::Hz(1000.0),
                                q: 1.0,
                                eq_type: eq::EqType::Peak,
                            },
                        ],
                    ),
                    (
                        String::from("still another preset"),
                        [
                            eq::Eq {
                                gain: eq::Gain::Db(0.0),
                                frequency: eq::Frequency::Hz(2000.0),
                                q: 2.0,
                                eq_type: eq::EqType::Notch,
                            },
                            eq::Eq {
                                gain: eq::Gain::Db(-12.0),
                                frequency: eq::Frequency::Hz(4000.0),
                                q: 0.3,
                                eq_type: eq::EqType::HighShelf,
                            },
                        ],
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
