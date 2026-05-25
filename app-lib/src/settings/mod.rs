use audio_lib::*;

pub mod ui;

#[derive(Debug, Clone)]
pub struct Settings<F: utils::Float, const NUM_BANDS: usize> {
    pub init_eqs: [eq::Eq<F>; NUM_BANDS],
    pub init_sample_rate: F,
    pub ui: ui::Settings<F>,
    pub persistence_dir: std::path::PathBuf,
}

impl<F: utils::Float, const NUM_BANDS: usize> Default for Settings<F, NUM_BANDS> {
    fn default() -> Self {
        let eq_ranges = ui::EqRanges::<F>::default();
        Self {
            init_eqs: Self::default_eqs(&eq_ranges.log_frequency_range),
            init_sample_rate: F::from(48000).unwrap(),
            ui: ui::Settings {
                eq_ranges: eq_ranges,
                impulse_response_params: ui::ImpulseResponseParams::default(),
                show_options: ui::ShowOptions::new_all_enabled(),
            },
            persistence_dir: dirs::config_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("sw")
                .join("eq_plotter"),
        }
    }
}

impl<F: utils::Float, const NUM_BANDS: usize> Settings<F, NUM_BANDS> {
    fn default_eqs(log_frequency_range: &std::ops::RangeInclusive<F>) -> [eq::Eq<F>; NUM_BANDS] {
        let log_frequency_step = (*log_frequency_range.end() - *log_frequency_range.start())
            / F::from(NUM_BANDS + 1).unwrap();
        let active_index = (F::from(NUM_BANDS).unwrap() / F::TWO).to_usize().unwrap();
        std::array::from_fn(|i| {
            let frequency = eq::Frequency::LogHz(
                *log_frequency_range.start() + F::from(i + 1).unwrap() * log_frequency_step,
            );
            eq::Eq {
                gain: eq::Gain::Db(F::from(3).unwrap()),
                frequency: frequency,
                q: F::from(0.7).unwrap(),
                eq_type: if i == active_index {
                    eq::EqType::Peak
                } else {
                    eq::EqType::Bypassed
                },
            }
        })
    }
}
