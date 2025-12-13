use audio_lib::*;
use eq_plotter_egui;
use nih_plug::prelude::*;
use std::sync::Arc;

// we have our own EqType here because we need nih-plugs Enum trait (can this be done simpler?)
#[derive(Enum, PartialEq)]
pub enum EqType {
    Volume,
    LowPass,
    HighPass,
    BandPass,
    AllPass,
    Notch,
    Peak,
    LowShelf,
    HighShelf,
}

impl From<eq::EqType> for EqType {
    fn from(eq: eq::EqType) -> Self {
        match eq {
            eq::EqType::Volume => EqType::Volume,
            eq::EqType::LowPass => EqType::LowPass,
            eq::EqType::HighPass => EqType::HighPass,
            eq::EqType::BandPass => EqType::BandPass,
            eq::EqType::AllPass => EqType::AllPass,
            eq::EqType::Notch => EqType::Notch,
            eq::EqType::Peak => EqType::Peak,
            eq::EqType::LowShelf => EqType::LowShelf,
            eq::EqType::HighShelf => EqType::HighShelf,
        }
    }
}
impl Into<eq::EqType> for EqType {
    fn into(self) -> eq::EqType {
        match self {
            EqType::Volume => eq::EqType::Volume,
            EqType::LowPass => eq::EqType::LowPass,
            EqType::HighPass => eq::EqType::HighPass,
            EqType::BandPass => eq::EqType::BandPass,
            EqType::AllPass => eq::EqType::AllPass,
            EqType::Notch => eq::EqType::Notch,
            EqType::Peak => eq::EqType::Peak,
            EqType::LowShelf => eq::EqType::LowShelf,
            EqType::HighShelf => eq::EqType::HighShelf,
        }
    }
}

#[derive(Params)]
pub struct EqPluginParams {
    #[persist = "editor-state"]
    editor_state: Arc<nih_plug_egui::EguiState>,

    #[id = "gain_db"]
    pub gain_db: FloatParam,

    #[id = "frequency"]
    pub frequency: FloatParam,

    #[id = "q"]
    pub q: FloatParam,

    #[id = "eq_type"]
    pub eq_type: EnumParam<EqType>,
}

impl EqPluginParams {
    const SMOOTHING_LENGTH_MS: f32 = 20.0;
}

impl Default for EqPluginParams {
    fn default() -> Self {
        Self {
            editor_state: nih_plug_egui::EguiState::from_size(1200, 800),
            gain_db: FloatParam::new(
                "Gain (dB)",
                0.0f32,
                FloatRange::Linear {
                    min: eq_plotter_egui::EqPlotter::MIN_GAIN_DB as f32,
                    max: eq_plotter_egui::EqPlotter::MAX_GAIN_DB as f32,
                },
            )
            .with_smoother(SmoothingStyle::Linear(Self::SMOOTHING_LENGTH_MS))
            .with_unit(" dB"),
            frequency: FloatParam::new(
                "frequency (Hz)",
                1000.0f32,
                FloatRange::Linear {
                    min: eq_plotter_egui::EqPlotter::MIN_FREQUENCY as f32,
                    max: eq_plotter_egui::EqPlotter::MAX_FREQUENCY as f32,
                },
            )
            .with_smoother(SmoothingStyle::Linear(Self::SMOOTHING_LENGTH_MS))
            .with_unit(" Hz"),
            q: FloatParam::new(
                "q",
                0.7f32,
                FloatRange::Linear {
                    min: eq_plotter_egui::EqPlotter::MIN_Q as f32,
                    max: eq_plotter_egui::EqPlotter::MAX_Q as f32,
                },
            )
            .with_smoother(SmoothingStyle::Linear(Self::SMOOTHING_LENGTH_MS)),
            eq_type: EnumParam::new("eq type", EqType::Peak),
        }
    }
}

pub struct EqPlugin {
    params: Arc<EqPluginParams>,
    filter: biquad::filter::Filter<f32>,
    sample_rate: f32,
}

impl Default for EqPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(EqPluginParams::default()),
            filter: biquad::filter::Filter::new(
                &biquad::coefficients::Coefficients::from_volume_db(0.0f32),
            ),
            sample_rate: 1.0,
        }
    }
}

impl Plugin for EqPlugin {
    const NAME: &'static str = "EqPlugin";
    const VENDOR: &'static str = "Stephan Widor";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "stephan@widor.online";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(1),
        main_output_channels: NonZeroU32::new(1),
        ..AudioIOLayout::const_default()
    }];

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = _buffer_config.sample_rate as f32;
        true
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let params = self.params.clone();
        let egui_state = params.editor_state.clone();
        nih_plug_egui::create_egui_editor(
            self.params.editor_state.clone(),
            (),
            |_, _| {},
            move |egui_ctx, setter, _state| {
                nih_plug_egui::resizable_window::ResizableWindow::new("res-wind")
                    .min_size(nih_plug_egui::egui::Vec2::new(1200.0, 800.0))
                    .show(egui_ctx, egui_state.as_ref(), |ui| {
                        let mut eq = eq::Eq {
                            gain_db: params.gain_db.value() as f64,
                            frequency: params.frequency.value() as f64,
                            q: params.q.value() as f64,
                            eq_type: params.eq_type.value().into(),
                        };

                        eq_plotter_egui::EqPlotter::draw(ui, &mut eq, 48000.0);

                        setter.begin_set_parameter(&params.gain_db);
                        setter.set_parameter(&params.gain_db, eq.gain_db as f32);
                        setter.end_set_parameter(&params.gain_db);

                        setter.begin_set_parameter(&params.frequency);
                        setter.set_parameter(&params.frequency, eq.frequency as f32);
                        setter.end_set_parameter(&params.frequency);

                        setter.begin_set_parameter(&params.q);
                        setter.set_parameter(&params.q, eq.q as f32);
                        setter.end_set_parameter(&params.q);

                        setter.begin_set_parameter(&params.eq_type);
                        setter.set_parameter(&params.eq_type, eq.eq_type.into());
                        setter.end_set_parameter(&params.eq_type);
                    });
            },
        )
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for channel_samples in buffer.iter_samples() {
            let eq = eq::Eq {
                gain_db: self.params.gain_db.smoothed.next(),
                frequency: self.params.frequency.smoothed.next(),
                q: self.params.q.smoothed.next(),
                eq_type: self.params.eq_type.value().into(),
            };
            self.filter.set_coefficients(
                biquad::coefficients::Coefficients::from_eq(&eq, self.sample_rate),
                false,
            );

            for sample in channel_samples {
                *sample = self.filter.process(*sample);
            }

            if self.params.editor_state.is_open() {
                // nothing to do yet
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for EqPlugin {
    const CLAP_ID: &'static str = "com.stephanwidor.EqPlugin";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("This is a simple Eq Plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Equalizer,
        ClapFeature::Mono,
    ];
}

impl Vst3Plugin for EqPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"widor.Eq__Plugin";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Eq];
}

nih_export_clap!(EqPlugin);
nih_export_vst3!(EqPlugin);
