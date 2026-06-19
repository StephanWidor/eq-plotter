use app_lib::settings::ui;
use audio_lib::*;

#[derive(Debug, Clone)]
pub struct Settings<F: utils::Float, const NUM_BANDS: usize> {
    pub init_eq: eq::MultibandEq<F, NUM_BANDS>,
    pub init_sample_rate: F,
    pub ui: ui::Settings<F>,
    pub persistence_dir: std::path::PathBuf,
}

impl<F: utils::Float, const NUM_BANDS: usize> Default for Settings<F, NUM_BANDS> {
    fn default() -> Self {
        let eq_ranges = ui::EqRanges::<F>::default();
        Self {
            init_eq: Self::default_eqs(&eq_ranges.log_frequency_range),
            init_sample_rate: F::from_integral(48000),
            ui: ui::Settings {
                eq_ranges: eq_ranges,
                impulse_response_params: ui::ImpulseResponseParams::default(),
                init_show_options: ui::ShowOptions::new_all_enabled(),
            },
            persistence_dir: dirs::config_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("sw")
                .join("eq_plotter"),
        }
    }
}

impl<F: utils::Float, const NUM_BANDS: usize> Settings<F, NUM_BANDS> {
    fn default_eqs(
        log_frequency_range: &std::ops::RangeInclusive<F>,
    ) -> eq::MultibandEq<F, NUM_BANDS> {
        let log_frequency_step = (*log_frequency_range.end() - *log_frequency_range.start())
            / F::from_integral(NUM_BANDS + 1);
        let active_index = (F::from_integral(NUM_BANDS) / F::TWO).to_usize().unwrap();
        eq::MultibandEq {
            eqs: std::array::from_fn(|i| {
                let frequency = eq::Frequency::LogHz(
                    *log_frequency_range.start() + F::from_integral(i + 1) * log_frequency_step,
                );
                eq::Eq {
                    gain: eq::Gain::Db(F::from_integral(3)),
                    frequency: frequency,
                    q: F::from_float(0.7),
                    eq_type: if i == active_index {
                        eq::EqType::Peak
                    } else {
                        eq::EqType::Bypassed
                    },
                    makeup_gain: eq::Gain::Amplitude(F::ONE),
                }
            }),
            processing_type: eq::MultibandType::Sequential,
        }
    }
}
