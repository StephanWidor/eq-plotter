// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync;//::{self, RwLock};
use app_lib as app;
use audio_lib::eq;

slint::include_modules!();

pub struct EqPlotter {
    ui: EqPlotterUi,
    eq: sync::Arc<sync::RwLock<eq::Eq<f64>>>,
}

impl EqPlotter {
    pub fn new() -> core::result::Result<Self, slint::PlatformError> {
        let eq_plotter=EqPlotter {
            ui: EqPlotterUi::new()?,
            eq: sync::Arc::new(sync::RwLock::new( app::DEFAULT_EQ)),
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
        let read_eq=self.eq.read().unwrap();
        let ui_eq=self.ui.global::<Eq>();

        let eq_type_strings:[slint::SharedString;eq::EqType::VARIANT_COUNT]=
            array_init::from_iter(eq::EqType::ALL.iter().map(|eq_type:&eq::EqType|{eq_type.to_string().into()})).unwrap();
        ui_eq.set_eq_types(slint::VecModel::from_slice(&eq_type_strings));
        ui_eq.set_eq_type(read_eq.eq_type.to_string().into());

        ui_eq.set_min_gain_db(app::MIN_GAIN_DB as f32);
        ui_eq.set_max_gain_db(app::MAX_GAIN_DB as f32);
        ui_eq.set_gain_db(read_eq.gain.db() as f32);

        ui_eq.set_min_frequency(app::MIN_FREQUENCY as f32);
        ui_eq.set_max_frequency(app::MAX_FREQUENCY as f32);
        ui_eq.set_min_log_frequency(app::MIN_LOG_FREQUENCY as f32);
        ui_eq.set_max_log_frequency(app::MAX_LOG_FREQUENCY as f32);
        ui_eq.set_frequency(read_eq.frequency.hz() as f32);
        ui_eq.set_log_frequency(read_eq.frequency.log_hz() as f32);

        ui_eq.set_min_q(app::MIN_Q as f32);
        ui_eq.set_max_q(app::MAX_Q as f32);
        ui_eq.set_q(read_eq.q as f32);
    }

    fn init_ui_callbacks(&self) {
        let ui_callbacks=self.ui.global::<Callbacks>();
        ui_callbacks.on_request_set_eq_type({
            let ui_handle = self.ui.as_weak().unwrap();
            let eq_handle=self.eq.clone();
            move |eq_type: slint::SharedString|{
                let new_eq_type_option=eq::EqType::try_from(eq_type.as_str());
                match new_eq_type_option {
                    Ok(new_eq_type) => {
                        println!("Setting eq type to {}", new_eq_type.to_string());
                        ui_handle.global::<Eq>().set_eq_type(new_eq_type.to_string().into());
                        eq_handle.write().unwrap().eq_type=new_eq_type;
                    }
                    _ => {    // we actually shouldn't ever get here!
                        ui_handle.global::<Eq>().set_eq_type(eq_handle.read().unwrap().eq_type.to_string().into());
                    }
                }
            }
        });
        ui_callbacks.on_request_set_gain_db({
            let ui_handle = self.ui.as_weak().unwrap();
            let eq_handle=self.eq.clone();
            move |gain_db: f32|{
                println!("Setting gain to {}dB", gain_db);
                ui_handle.global::<Eq>().set_gain_db(gain_db);
                eq_handle.write().unwrap().gain=eq::Gain::Db(gain_db as f64);
            }
        });
        ui_callbacks.on_request_set_log_frequency({
            let ui_handle = self.ui.as_weak().unwrap();
            let eq_handle=self.eq.clone();
            move |log_frequency: f32|{
                let frequency=eq::Frequency::LogHz(log_frequency as f64);

                println!("Setting frequency to {}Hz (log: {})", frequency.hz(), frequency.log_hz());
                let ui_eq=ui_handle.global::<Eq>();
                ui_eq.set_log_frequency(frequency.hz() as f32);
                ui_eq.set_frequency(frequency.hz() as f32);
                eq_handle.write().unwrap().frequency=frequency;
            }
        });
        ui_callbacks.on_request_set_q({
            let ui_handle = self.ui.as_weak().unwrap();
            let eq_handle=self.eq.clone();
            move |q: f32|{
                println!("Setting q to {}", q);
                ui_handle.global::<Eq>().set_q(q);
                eq_handle.write().unwrap().q=q as f64;
            }
        });
    }
}
