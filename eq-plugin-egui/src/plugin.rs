use crate::*;
use audio_lib::*;
use nih::Plugin as NihPlugin;
use nih_plug::prelude as nih;
use std::sync;
use std::sync::atomic::Ordering;

pub struct Plugin {
    params: sync::Arc<params::PluginParams>,
    processor: processor::Processor<{ params::MAX_NUM_CHANNELS }, { params::NUM_BANDS }>,
    analyzer: fft::SignalAnalyzer<f32, { params::ANALYZER_NUM_BINS }, { params::MAX_NUM_CHANNELS }>,
}

impl Default for Plugin {
    fn default() -> Self {
        Self {
            params: sync::Arc::new(params::PluginParams::default()),
            processor: processor::Processor::default(),
            analyzer: fft::SignalAnalyzer::new(&params::DEFAULT_ANALYZER_COEFFICIENTS),
        }
    }
}

const AUDIO_LAYOUTS: [nih::AudioIOLayout; params::MAX_NUM_CHANNELS] = {
    const MAX_NUM_CHANNELS: usize = params::MAX_NUM_CHANNELS;
    // seems like std::array::from_fn doesn't work as const :-(
    let mut layouts: [nih::AudioIOLayout; MAX_NUM_CHANNELS] =
        [nih::AudioIOLayout::const_default(); MAX_NUM_CHANNELS];
    let mut i = 0;
    while i < MAX_NUM_CHANNELS {
        let num_channels = (MAX_NUM_CHANNELS - i) as u32;
        layouts[i].main_input_channels = nih::NonZeroU32::new(num_channels);
        layouts[i].main_output_channels = nih::NonZeroU32::new(num_channels);
        i += 1;
    }
    layouts
};

impl nih::Plugin for Plugin {
    const NAME: &'static str = "EqPlugin";
    const VENDOR: &'static str = "Stephan Widor";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "stephan@widor.online";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [nih::AudioIOLayout] = &AUDIO_LAYOUTS;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> sync::Arc<dyn nih::Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &nih::AudioIOLayout,
        _buffer_config: &nih::BufferConfig,
        _context: &mut impl nih::InitContext<Self>,
    ) -> bool {
        let sample_rate = _buffer_config.sample_rate as f32;
        self.params
            .sample_rate
            .store(sample_rate, std::sync::atomic::Ordering::Relaxed);

        if sample_rate
            != self
                .params
                .analyzer_data
                .frequency_bins
                .read()
                .unwrap()
                .sample_rate()
        {
            self.params.analyzer_data.reset(sample_rate);
        }

        self.processor.initialize(&self.params.eqs(), sample_rate)
    }

    fn editor(
        &mut self,
        _async_executor: nih::AsyncExecutor<Self>,
    ) -> Option<Box<dyn nih::Editor>> {
        editor::create_editor(self.params.clone())
    }

    fn process(
        &mut self,
        buffer: &mut nih::Buffer,
        _aux: &mut nih::AuxiliaryBuffers,
        _context: &mut impl nih::ProcessContext<Self>,
    ) -> nih::ProcessStatus {
        self.processor.process(
            &self.params.eqs(),
            self.params.sample_rate.load(Ordering::Relaxed),
            buffer.as_slice(),
        );
        if self
            .params
            .show_signal_gain_spectrum
            .load(Ordering::Relaxed)
        {
            let analyzer_data = &self.params.analyzer_data;
            self.analyzer.push(
                buffer.as_slice_immutable(),
                &analyzer_data.frequency_bins.read().unwrap(),
                &analyzer_data.linear_gains.producer,
            );
        }
        nih::ProcessStatus::Normal
    }
}

impl nih::ClapPlugin for Plugin {
    const CLAP_ID: &'static str = "com.stephanwidor.EqPlugin";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("This is a simple Eq Plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [nih::ClapFeature] = &[
        nih::ClapFeature::AudioEffect,
        nih::ClapFeature::Equalizer,
        nih::ClapFeature::Mono,
    ];
}

impl nih::Vst3Plugin for Plugin {
    const VST3_CLASS_ID: [u8; 16] = *b"widor.Eq__Plugin";
    const VST3_SUBCATEGORIES: &'static [nih::Vst3SubCategory] =
        &[nih::Vst3SubCategory::Fx, nih::Vst3SubCategory::Eq];
}

nih::nih_export_clap!(Plugin);
nih::nih_export_vst3!(Plugin);
