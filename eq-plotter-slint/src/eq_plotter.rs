use app_lib as app;
use audio_lib::eq;
use std::sync;

slint::include_modules!();

pub struct EqPlotter {
    ui: EqPlotterUi,
    eq: sync::Arc<sync::RwLock<eq::Eq<f32>>>,
    background_color: slint::Color,
    sample_rate: f32,
}

impl EqPlotter {
    pub fn new() -> core::result::Result<Self, slint::PlatformError> {
        let ui = EqPlotterUi::new()?;
        let background_color = ui.global::<Colors>().get_background_color();

        let eq_plotter = EqPlotter {
            ui: ui,
            eq: sync::Arc::new(sync::RwLock::new(app::DEFAULT_EQ.into())),
            background_color: background_color,
            sample_rate: 48000.0f32,
        };

        eq_plotter.init_ui_properties();
        eq_plotter.init_ui_callbacks();

        Ok(eq_plotter)
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.ui.run()?;
        Ok(())
    }

    fn init_ui_properties(&self) {
        let read_eq = self.eq.read().unwrap();
        let ui_eq = self.ui.global::<Eq>();

        let eq_type_strings: [slint::SharedString; eq::EqType::VARIANT_COUNT] =
            array_init::from_iter(
                eq::EqType::ALL
                    .iter()
                    .map(|eq_type: &eq::EqType| eq_type.to_string().into()),
            )
            .unwrap();
        ui_eq.set_eq_types(slint::VecModel::from_slice(&eq_type_strings));
        ui_eq.set_eq_type(read_eq.eq_type.to_string().into());

        ui_eq.set_min_gain_db(app::MIN_GAIN_DB as f32);
        ui_eq.set_max_gain_db(app::MAX_GAIN_DB as f32);
        ui_eq.set_gain_db(read_eq.gain.db());

        ui_eq.set_min_frequency(app::MIN_FREQUENCY as f32);
        ui_eq.set_max_frequency(app::MAX_FREQUENCY as f32);
        ui_eq.set_min_log_frequency(app::MIN_LOG_FREQUENCY as f32);
        ui_eq.set_max_log_frequency(app::MAX_LOG_FREQUENCY as f32);
        ui_eq.set_frequency(read_eq.frequency.hz());
        ui_eq.set_log_frequency(read_eq.frequency.log_hz());

        ui_eq.set_min_q(app::MIN_Q as f32);
        ui_eq.set_max_q(app::MAX_Q as f32);
        ui_eq.set_q(read_eq.q);

        self.ui
            .set_gain_control_visible(read_eq.eq_type.has_gain_db());
        self.ui
            .set_frequency_control_visible(read_eq.eq_type.has_frequency());
        self.ui.set_q_control_visible(read_eq.eq_type.has_q());
    }

    fn init_ui_callbacks(&self) {
        let ui_callbacks = self.ui.global::<Callbacks>();
        ui_callbacks.on_request_set_eq_type({
            let ui_handle = self.ui.as_weak().unwrap();
            let eq_handle = self.eq.clone();
            move |eq_type: slint::SharedString| {
                let new_eq_type_option = eq::EqType::try_from(eq_type.as_str());
                match new_eq_type_option {
                    Ok(new_eq_type) => {
                        ui_handle
                            .global::<Eq>()
                            .set_eq_type(new_eq_type.to_string().into());
                        eq_handle.write().unwrap().eq_type = new_eq_type;
                        ui_handle.set_gain_control_visible(new_eq_type.has_gain_db());
                        ui_handle.set_frequency_control_visible(new_eq_type.has_frequency());
                        ui_handle.set_q_control_visible(new_eq_type.has_q());
                    }
                    _ => {
                        // we actually shouldn't ever get here!
                        ui_handle
                            .global::<Eq>()
                            .set_eq_type(eq_handle.read().unwrap().eq_type.to_string().into());
                    }
                }
            }
        });
        ui_callbacks.on_request_set_gain_db({
            let ui_handle = self.ui.as_weak().unwrap();
            let eq_handle = self.eq.clone();
            move |gain_db: f32| {
                ui_handle.global::<Eq>().set_gain_db(gain_db);
                eq_handle.write().unwrap().gain = eq::Gain::Db(gain_db);
            }
        });
        ui_callbacks.on_request_set_log_frequency({
            let ui_handle = self.ui.as_weak().unwrap();
            let eq_handle = self.eq.clone();
            move |log_frequency: f32| {
                let frequency = eq::Frequency::LogHz(log_frequency);

                let ui_eq = ui_handle.global::<Eq>();
                ui_eq.set_log_frequency(frequency.hz());
                ui_eq.set_frequency(frequency.hz());
                eq_handle.write().unwrap().frequency = frequency;
            }
        });
        ui_callbacks.on_request_set_q({
            let ui_handle = self.ui.as_weak().unwrap();
            let eq_handle = self.eq.clone();
            move |q: f32| {
                ui_handle.global::<Eq>().set_q(q);
                eq_handle.write().unwrap().q = q;
            }
        });
        ui_callbacks.on_render_eq_plots({
            let sample_rate = self.sample_rate;
            let eq_handle = self.eq.clone();
            let background_color = self.background_color;
            move || {
                crate::plotters::render_eq_plots(
                    &eq_handle.read().unwrap(),
                    sample_rate,
                    background_color,
                )
            }
        });
    }
}
