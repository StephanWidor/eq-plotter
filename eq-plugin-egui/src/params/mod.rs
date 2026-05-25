use crate::*;
use audio_lib::*;
use egui_lib::colors;
use std::sync::{self, atomic};

pub mod eq_params;
pub mod eq_type;
pub mod show_params;

pub use eq_params::EqParams;
pub use show_params::ShowParams;

// hm, can we somehow get rid of this without destroying the nice::Enum and nice::Params derive?
use nice_plug::params::Params;

#[derive(nice::Params)]
pub struct PluginParams<
    const NUM_BANDS: usize,
    const NUM_CHANNELS: usize,
    const ANALYZER_NUM_BINS: usize,
> {
    #[persist = "editor_state"]
    pub editor_state: sync::Arc<nice_plug_egui::EguiState>,

    #[nested(array, group = "eq_params")]
    pub eq_params: [EqParams; NUM_BANDS],

    pub sample_rate: nice::AtomicF32,

    #[nested(group = "show_params")]
    pub show_params: ShowParams,

    pub drag_eq_index: atomic::AtomicUsize,
    pub analyzer_data:
        fft::signal_analyzer::SharedData<f32, { ANALYZER_NUM_BINS }, { NUM_CHANNELS }>,
    pub eq_ranges: EqRanges,
    pub impulse_response_params: ImpulseResponseParams,
    pub color_palette: colors::ColorPalette,
}

impl<const NUM_BANDS: usize, const NUM_CHANNELS: usize, const ANALYZER_NUM_BINS: usize>
    PluginParams<NUM_BANDS, NUM_CHANNELS, ANALYZER_NUM_BINS>
{
    pub fn new(settings: &Settings<NUM_BANDS>, smoothing_length_ms: f32) -> Self {
        let eq_ranges = settings.ui.eq_ranges.clone();
        Self {
            editor_state: nice_plug_egui::EguiState::from_size(1000, 700),
            eq_params: std::array::from_fn(|index| {
                EqParams::from_eq(
                    format!(" [{}]", index + 1).as_str(),
                    &settings.init_eqs[index],
                    &eq_ranges.log_frequency_range,
                    &eq_ranges.db_range,
                    &eq_ranges.q_range,
                    smoothing_length_ms,
                )
            }),
            sample_rate: nice::AtomicF32::new(settings.init_sample_rate),
            show_params: ShowParams::from_options(&settings.ui.show_options),
            drag_eq_index: atomic::AtomicUsize::new(usize::MAX),
            analyzer_data: fft::signal_analyzer::SharedData::new(settings.init_sample_rate),
            eq_ranges: eq_ranges,
            impulse_response_params: settings.ui.impulse_response_params.clone(),
            color_palette: colors::ColorPalette::default(),
        }
    }

    pub fn eqs<F: utils::Float>(&self) -> [eq::Eq<F>; NUM_BANDS] {
        std::array::from_fn(|index| self.eq_params[index].to_eq())
    }
}
