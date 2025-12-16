use audio_lib::*;
use eq_plotter_egui::EqPlotter;
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
                "gain (dB)",
                EqPlotter::DEFAULT_EQ.gain_db as f32,
                FloatRange::Linear {
                    min: EqPlotter::MIN_GAIN_DB as f32,
                    max: EqPlotter::MAX_GAIN_DB as f32,
                },
            )
            .with_smoother(SmoothingStyle::Linear(Self::SMOOTHING_LENGTH_MS))
            .with_unit(" dB"),
            frequency: FloatParam::new(
                "frequency (Hz)",
                EqPlotter::DEFAULT_EQ.frequency as f32,
                FloatRange::Linear {
                    min: EqPlotter::MIN_FREQUENCY as f32,
                    max: EqPlotter::MAX_FREQUENCY as f32,
                },
            )
            .with_smoother(SmoothingStyle::Linear(Self::SMOOTHING_LENGTH_MS))
            .with_unit(" Hz"),
            q: FloatParam::new(
                "q",
                EqPlotter::DEFAULT_EQ.q as f32,
                FloatRange::Linear {
                    min: EqPlotter::MIN_Q as f32,
                    max: EqPlotter::MAX_Q as f32,
                },
            )
            .with_smoother(SmoothingStyle::Linear(Self::SMOOTHING_LENGTH_MS)),
            eq_type: EnumParam::new("eq type", EqPlotter::DEFAULT_EQ.eq_type.into()),
        }
    }
}

pub struct EqPlugin {
    params: Arc<EqPluginParams>,
    sample_rate: Arc<AtomicF32>,
    eq: eq::Eq<f32>,
    filter: biquad::filter::Filter<f32>,
}

impl EqPlugin {
    const INIT_FILTER_COEFFICIENTS: biquad::coefficients::Coefficients<f32> =
        biquad::coefficients::Coefficients {
            a1: 0.0,
            a2: 0.0,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
        };
    const INIT_EQ: eq::Eq<f32> = eq::Eq {
        gain_db: std::f32::NEG_INFINITY,
        frequency: 0.0,
        q: 0.0,
        eq_type: eq::EqType::Volume,
    };
}

impl Default for EqPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(EqPluginParams::default()),
            sample_rate: Arc::new(AtomicF32::new(1.0)),
            eq: Self::INIT_EQ,
            filter: biquad::filter::Filter::new(&Self::INIT_FILTER_COEFFICIENTS),
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
        self.sample_rate.store(
            _buffer_config.sample_rate as f32,
            std::sync::atomic::Ordering::Relaxed,
        );

        self.eq = Self::INIT_EQ;
        self.filter
            .set_coefficients(Self::INIT_FILTER_COEFFICIENTS, true);
        true
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let params = self.params.clone();
        let egui_state = params.editor_state.clone();
        let sample_rate = self.sample_rate.clone();
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
                        EqPlotter::draw(
                            ui,
                            &mut eq,
                            sample_rate.load(std::sync::atomic::Ordering::Relaxed) as f64,
                        );

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
        assert!(buffer.channels() == 1); // we are always mono
        for channel_samples in buffer.iter_samples() {
            let eq = eq::Eq {
                gain_db: self.params.gain_db.smoothed.next(),
                frequency: self.params.frequency.smoothed.next(),
                q: self.params.q.smoothed.next(),
                eq_type: self.params.eq_type.value().into(),
            };
            if eq != self.eq {
                self.eq = eq;
                let new_coefficients = biquad::coefficients::Coefficients::from_eq(
                    &self.eq,
                    self.sample_rate.load(std::sync::atomic::Ordering::Relaxed),
                );
                if biquad::utils::is_stable(&new_coefficients) {
                    self.filter.set_coefficients(new_coefficients, false);
                }
            }

            for sample in channel_samples {
                *sample = self.filter.process(*sample);
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
