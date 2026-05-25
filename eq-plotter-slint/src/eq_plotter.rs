use crate::*;
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
    pub fn new(settings: &Settings) -> core::result::Result<Self, slint::PlatformError> {
        let ui = EqPlotterUi::new()?;
        let background_color = ui.global::<Colors>().get_background_color();

        let eq_plotter = EqPlotter {
            ui: ui,
            eq: sync::Arc::new(sync::RwLock::new(settings.init_eqs[0].clone())),
            background_color: background_color,
            sample_rate: settings.init_sample_rate,
        };

        eq_plotter.init_ui_properties(&settings.ui.eq_ranges);
        eq_plotter.init_ui_callbacks(
            settings.ui.eq_ranges.clone(),
            settings.ui.impulse_response_params.clone(),
        );

        Ok(eq_plotter)
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.ui.run()?;
        Ok(())
    }

    fn init_ui_properties(&self, eq_ranges: &EqRanges) {
        let read_eq = self.eq.read().unwrap();
        let ui_eq = self.ui.global::<Eq>();

        let eq_type_strings: [slint::SharedString; eq::EqType::VARIANT_COUNT] =
            std::array::from_fn(|i| eq::EqType::ALL[i].to_string().into());
        ui_eq.set_eq_types(slint::VecModel::from_slice(&eq_type_strings));
        ui_eq.set_eq_type(read_eq.eq_type.to_string().into());

        ui_eq.set_min_gain_db(*eq_ranges.db_range.start());
        ui_eq.set_max_gain_db(*eq_ranges.db_range.end());
        ui_eq.set_gain_db(read_eq.gain.db());

        let frequency_range = eq_ranges.frequency_range();
        ui_eq.set_min_frequency(*frequency_range.start());
        ui_eq.set_max_frequency(*frequency_range.end());
        ui_eq.set_min_log_frequency(*eq_ranges.log_frequency_range.start());
        ui_eq.set_max_log_frequency(*eq_ranges.log_frequency_range.end());
        ui_eq.set_frequency(read_eq.frequency.hz());
        ui_eq.set_log_frequency(read_eq.frequency.log_hz());

        ui_eq.set_min_q(*eq_ranges.q_range.start());
        ui_eq.set_max_q(*eq_ranges.q_range.end());
        ui_eq.set_q(read_eq.q);

        self.ui
            .set_gain_control_visible(read_eq.eq_type.has_gain_db());
        self.ui
            .set_frequency_control_visible(read_eq.eq_type.has_frequency());
        self.ui.set_q_control_visible(read_eq.eq_type.has_q());
    }

    fn init_ui_callbacks(
        &self,
        eq_ranges: EqRanges,
        impulse_response_params: ImpulseResponseParams,
    ) {
        let ui_callbacks = self.ui.global::<Callbacks>();
        ui_callbacks.on_request_set_eq_type({
            let ui_ptr = self.ui.as_weak();
            let eq = self.eq.clone();
            move |eq_type: slint::SharedString| {
                let new_eq_type_option = eq::EqType::try_from(eq_type.as_str());
                let ui = ui_ptr.unwrap();
                match new_eq_type_option {
                    Ok(new_eq_type) => {
                        ui.global::<Eq>()
                            .set_eq_type(new_eq_type.to_string().into());
                        eq.write().unwrap().eq_type = new_eq_type;
                        ui.set_gain_control_visible(new_eq_type.has_gain_db());
                        ui.set_frequency_control_visible(new_eq_type.has_frequency());
                        ui.set_q_control_visible(new_eq_type.has_q());
                    }
                    _ => {
                        // we actually shouldn't ever get here!
                        ui.global::<Eq>()
                            .set_eq_type(eq.read().unwrap().eq_type.to_string().into());
                    }
                }
            }
        });
        ui_callbacks.on_request_set_gain_db({
            let ui_ptr = self.ui.as_weak();
            let eq = self.eq.clone();
            move |gain_db: f32| {
                let ui = ui_ptr.unwrap();
                ui.global::<Eq>().set_gain_db(gain_db);
                eq.write().unwrap().gain = eq::Gain::Db(gain_db);
            }
        });
        ui_callbacks.on_request_set_log_frequency({
            let ui_ptr = self.ui.as_weak();
            let eq = self.eq.clone();
            move |log_frequency: f32| {
                let ui = ui_ptr.unwrap();
                let frequency = eq::Frequency::LogHz(log_frequency);
                let ui_eq = ui.global::<Eq>();
                ui_eq.set_log_frequency(frequency.hz());
                ui_eq.set_frequency(frequency.hz());
                eq.write().unwrap().frequency = frequency;
            }
        });
        ui_callbacks.on_request_set_q({
            let ui_ptr = self.ui.as_weak();
            let eq = self.eq.clone();
            move |q: f32| {
                let ui = ui_ptr.unwrap();
                ui.global::<Eq>().set_q(q);
                eq.write().unwrap().q = q;
            }
        });
        ui_callbacks.on_render_eq_plots({
            let eq = self.eq.clone();
            let sample_rate = self.sample_rate;
            let background_color = self.background_color;
            move |width, height| {
                crate::plotters::render::render_eq_plots(
                    &eq.read().unwrap(),
                    sample_rate,
                    &eq_ranges,
                    &impulse_response_params,
                    width as u32,
                    height as u32,
                    background_color,
                )
            }
        });
    }
}
