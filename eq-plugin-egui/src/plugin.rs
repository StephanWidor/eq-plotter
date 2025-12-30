use crate::params;
use app_lib as app;
use audio_lib::*;
use eq_plotter_egui;
use nih_plug::prelude::*;
use std::sync;

pub struct EqPlugin {
    params: sync::Arc<params::EqPluginParams>,
    sample_rate: sync::Arc<AtomicF32>,
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
        gain: eq::Gain::Db(std::f32::NEG_INFINITY),
        frequency: eq::Frequency::LogHz(std::f32::NEG_INFINITY),
        q: 0.0,
        eq_type: eq::EqType::Volume,
    };
}

impl Default for EqPlugin {
    fn default() -> Self {
        Self {
            params: sync::Arc::new(params::EqPluginParams::default()),
            sample_rate: sync::Arc::new(AtomicF32::new(1.0)),
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

    fn params(&self) -> sync::Arc<dyn Params> {
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
        let sample_rate = self.sample_rate.clone();
        let editor_size = params.editor_state.size();
        nih_plug_egui::create_egui_editor(
            self.params.editor_state.clone(),
            (),
            move |egui_ctx, _| {
                egui_ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::Vec2::new(
                    editor_size.0 as f32,
                    editor_size.1 as f32,
                )));
                egui_ctx.send_viewport_cmd(egui::ViewportCommand::Resizable(false));
            },
            move |egui_ctx, setter, _state| {
                egui::CentralPanel::default()
                    .frame(
                        egui::Frame::default()
                            .inner_margin(20)
                            .fill(egui::Color32::from_rgb(
                                app::UI_BACKGROUND_COLOR[0],
                                app::UI_BACKGROUND_COLOR[1],
                                app::UI_BACKGROUND_COLOR[2],
                            )),
                    )
                    .show(egui_ctx, |ui| {
                        let eq = eq::Eq {
                            gain: eq::Gain::Db(params.gain_db.value() as f64),
                            frequency: eq::Frequency::LogHz(params.log_frequency.value() as f64),
                            q: params.q.value() as f64,
                            eq_type: params.eq_type.value().into(),
                        };
                        let new_eq = eq_plotter_egui::EqPlotter::draw(
                            ui,
                            &eq,
                            sample_rate.load(std::sync::atomic::Ordering::Relaxed) as f64,
                        );

                        if eq == new_eq {
                            return; // no changes
                        }

                        setter.begin_set_parameter(&params.gain_db);
                        setter.begin_set_parameter(&params.log_frequency);
                        setter.begin_set_parameter(&params.q);
                        setter.begin_set_parameter(&params.eq_type);
                        setter.set_parameter(&params.gain_db, new_eq.gain.db() as f32);
                        setter
                            .set_parameter(&params.log_frequency, new_eq.frequency.log_hz() as f32);
                        setter.set_parameter(&params.q, new_eq.q as f32);
                        setter.set_parameter(&params.eq_type, new_eq.eq_type.into());
                        setter.end_set_parameter(&params.gain_db);
                        setter.end_set_parameter(&params.log_frequency);
                        setter.end_set_parameter(&params.q);
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
                gain: eq::Gain::Db(self.params.gain_db.smoothed.next()),
                frequency: eq::Frequency::LogHz(self.params.log_frequency.smoothed.next()),
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
