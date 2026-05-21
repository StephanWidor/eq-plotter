use crate::*;
use audio_lib::*;
use nice::Plugin as NicePlugin;
use nice_plug::prelude as nice;
use std::sync;
use std::sync::atomic::Ordering;

pub struct Plugin {
    params: sync::Arc<params::PluginParams>,
    processor: processor::Processor<{ config::MAX_NUM_CHANNELS }, { config::NUM_BANDS }>,
    analyzer: fft::SignalAnalyzer<f32, { config::ANALYZER_NUM_BINS }, { config::MAX_NUM_CHANNELS }>,
}

impl Default for Plugin {
    fn default() -> Self {
        Self {
            params: sync::Arc::new(params::PluginParams::default()),
            processor: processor::Processor::default(),
            analyzer: fft::SignalAnalyzer::new(&config::DEFAULT_ANALYZER_COEFFICIENTS),
        }
    }
}

const AUDIO_LAYOUTS: [nice::AudioIOLayout; config::MAX_NUM_CHANNELS] = {
    const MAX_NUM_CHANNELS: usize = config::MAX_NUM_CHANNELS;
    // seems like std::array::from_fn doesn't work as const :-(
    let mut layouts: [nice::AudioIOLayout; MAX_NUM_CHANNELS] =
        [nice::AudioIOLayout::const_default(); MAX_NUM_CHANNELS];
    let mut i = 0;
    while i < MAX_NUM_CHANNELS {
        let num_channels = (MAX_NUM_CHANNELS - i) as u32;
        layouts[i].main_input_channels = nice::NonZeroU32::new(num_channels);
        layouts[i].main_output_channels = nice::NonZeroU32::new(num_channels);
        i += 1;
    }
    layouts
};

impl nice::Plugin for Plugin {
    const NAME: &'static str = "EqPlugin";
    const VENDOR: &'static str = "Stephan Widor";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "stephan@widor.online";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [nice::AudioIOLayout] = &AUDIO_LAYOUTS;

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
        _async_executor: nice::AsyncExecutor<Self>,
    ) -> Option<Box<dyn nice::Editor>> {
        editor::create_editor(self.params.clone())
    }

    fn process(
        &mut self,
        buffer: &mut nice::Buffer,
        _aux: &mut nice::AuxiliaryBuffers,
        _context: &mut impl nice::ProcessContext<Self>,
    ) -> nice::ProcessStatus {
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
        nice::ProcessStatus::Normal
    }
}

impl nice::ClapPlugin for Plugin {
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

impl nice::Vst3Plugin for Plugin {
    const VST3_CLASS_ID: [u8; 16] = *b"widor.Eq__Plugin";
    const VST3_SUBCATEGORIES: &'static [nice::Vst3SubCategory] =
        &[nice::Vst3SubCategory::Fx, nice::Vst3SubCategory::Eq];
}

nice::nice_export_clap!(Plugin);
nice::nice_export_vst3!(Plugin);
