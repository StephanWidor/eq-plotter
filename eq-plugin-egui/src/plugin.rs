use crate::*;
use app_lib::persistence;
use nice::Plugin as NicePlugin;
use std::sync::{self, atomic};

pub struct Plugin<const NUM_BANDS: usize, const NUM_CHANNELS: usize, const ANALYZER_NUM_BINS: usize>
{
    params: sync::Arc<params::PluginParams<NUM_BANDS, NUM_CHANNELS, ANALYZER_NUM_BINS>>,
    presets: sync::Arc<sync::Mutex<Presets<NUM_BANDS>>>,
    processor: processor::Processor<{ NUM_BANDS }, { NUM_CHANNELS }, { ANALYZER_NUM_BINS }>,
    analyzer: analyzer::Analyzer<{ NUM_BANDS }, { NUM_CHANNELS }, { ANALYZER_NUM_BINS }>,
    ui_settings: UiSettings,
    persistence_dir: std::path::PathBuf,
}

impl<const NUM_BANDS: usize, const NUM_CHANNELS: usize, const ANALYZER_NUM_BINS: usize>
    Plugin<NUM_BANDS, NUM_CHANNELS, ANALYZER_NUM_BINS>
{
    pub fn new(
        app_settings: &AppSettings<NUM_BANDS>,
        analyzer_coefficients: &analyzer::Coefficients,
        smoothing_length_ms: f32,
        color_palette: egui_lib::colors::ColorPalette,
    ) -> Self {
        let params = sync::Arc::new(params::PluginParams::new(app_settings, smoothing_length_ms));
        let presets = if let Some(presets) = persistence::create_from_json_file::<Presets<NUM_BANDS>>(
            &Self::presets_file_path(&app_settings.persistence_dir).as_path(),
        ) {
            presets
        } else {
            Presets::<NUM_BANDS>::new()
        };
        Self {
            params: params.clone(),
            presets: sync::Arc::new(sync::Mutex::new(presets)),
            processor: processor::Processor::new(params.clone()),
            analyzer: analyzer::Analyzer::new(params.clone(), analyzer_coefficients),
            ui_settings: UiSettings {
                app: app_settings.ui.clone(),
                color_palette: color_palette,
            },
            persistence_dir: app_settings.persistence_dir.clone(),
        }
    }

    const AUDIO_IO_LAYOUTS_INSTANCE: [nice::AudioIOLayout; NUM_CHANNELS] =
        Self::make_audio_layouts();

    const fn make_audio_layouts() -> [nice::AudioIOLayout; NUM_CHANNELS] {
        // seems like std::array::from_fn doesn't work as const :-(
        let mut layouts: [nice::AudioIOLayout; NUM_CHANNELS] =
            [nice::AudioIOLayout::const_default(); NUM_CHANNELS];
        let mut i = 0;
        while i < NUM_CHANNELS {
            let num_channels = (NUM_CHANNELS - i) as u32;
            layouts[i].main_input_channels = nice::NonZeroU32::new(num_channels);
            layouts[i].main_output_channels = nice::NonZeroU32::new(num_channels);
            i += 1;
        }
        layouts
    }

    fn presets_file_path(presets_dir: &std::path::PathBuf) -> std::path::PathBuf {
        presets_dir.join("presets.json")
    }
}

impl<const NUM_BANDS: usize, const NUM_CHANNELS: usize, const ANALYZER_NUM_BINS: usize> Drop
    for Plugin<NUM_BANDS, NUM_CHANNELS, ANALYZER_NUM_BINS>
{
    fn drop(&mut self) {
        let presets = self.presets.lock().unwrap();
        persistence::save_to_json_file::<Presets<NUM_BANDS>>(
            &presets,
            &Self::presets_file_path(&self.persistence_dir),
        );
    }
}

impl<const NUM_BANDS: usize, const NUM_CHANNELS: usize, const ANALYZER_NUM_BINS: usize> Default
    for Plugin<NUM_BANDS, NUM_CHANNELS, ANALYZER_NUM_BINS>
{
    fn default() -> Self {
        Self::new(
            &AppSettings::<NUM_BANDS>::default(),
            &analyzer::Coefficients::default(),
            20_f32,
            egui_lib::colors::ColorPalette::default(),
        )
    }
}

impl<const NUM_BANDS: usize, const NUM_CHANNELS: usize, const ANALYZER_NUM_BINS: usize> nice::Plugin
    for Plugin<NUM_BANDS, NUM_CHANNELS, ANALYZER_NUM_BINS>
{
    const NAME: &'static str = "EqPlugin";
    const VENDOR: &'static str = "Stephan Widor";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "stephan@widor.online";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [nice::AudioIOLayout] = &Self::AUDIO_IO_LAYOUTS_INSTANCE;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> sync::Arc<dyn nice::Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &nice::AudioIOLayout,
        _buffer_config: &nice::BufferConfig,
        _context: &mut impl nice::InitContext<Self>,
    ) -> bool {
        let sample_rate = _buffer_config.sample_rate as f32;
        self.params
            .sample_rate
            .store(sample_rate, atomic::Ordering::Relaxed);
        self.processor.initialize() && self.analyzer.initialize()
    }

    fn process(
        &mut self,
        buffer: &mut nice::Buffer,
        _aux: &mut nice::AuxiliaryBuffers,
        _context: &mut impl nice::ProcessContext<Self>,
    ) -> nice::ProcessStatus {
        self.processor.process(buffer);
        self.analyzer.process(buffer);
        nice::ProcessStatus::Normal
    }

    fn editor(
        &mut self,
        _async_executor: nice::AsyncExecutor<Self>,
    ) -> Option<Box<dyn nice::Editor>> {
        editor::create_editor(
            self.params.clone(),
            self.presets.clone(),
            self.ui_settings.clone(),
        )
    }
}

impl<const NUM_BANDS: usize, const NUM_CHANNELS: usize, const ANALYZER_NUM_BINS: usize>
    nice::ClapPlugin for Plugin<NUM_BANDS, NUM_CHANNELS, ANALYZER_NUM_BINS>
{
    const CLAP_ID: &'static str = "com.stephanwidor.EqPlugin";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("This is a simple Eq Plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [nice::ClapFeature] = &[
        nice::ClapFeature::AudioEffect,
        nice::ClapFeature::Equalizer,
        nice::ClapFeature::Mono,
    ];
}

impl<const NUM_BANDS: usize, const NUM_CHANNELS: usize, const ANALYZER_NUM_BINS: usize>
    nice::Vst3Plugin for Plugin<NUM_BANDS, NUM_CHANNELS, ANALYZER_NUM_BINS>
{
    const VST3_CLASS_ID: [u8; 16] = *b"widor.Eq__Plugin";
    const VST3_SUBCATEGORIES: &'static [nice::Vst3SubCategory] =
        &[nice::Vst3SubCategory::Fx, nice::Vst3SubCategory::Eq];
}
