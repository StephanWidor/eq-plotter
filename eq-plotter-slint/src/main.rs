// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync;
use app_lib as app;
use audio_lib::eq;

slint::include_modules!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let eq=std::sync::Arc::new(sync::RwLock::new( app::DEFAULT_EQ));
    let eq_types=slint::VecModel::from_slice(&eq::EqType::ALL);

    let ui = EqPlotterUi::new()?;



    let ui_eq=ui.global::<Eq>();
    ui_eq.set_eq_types(slint::ModelRc::new(eq_types.clone().map(|eq_type| slint::SharedString::from(eq_type.to_string()))));

    ui_eq.set_min_gain_db(app::MIN_GAIN_DB as f32);
    ui_eq.set_max_gain_db(app::MAX_GAIN_DB as f32);
    ui_eq.set_gain_db(eq.write().unwrap().gain.db() as f32);
    ui.global::<Callbacks>().on_request_set_gain_db({
        let ui_handle = ui.as_weak().unwrap();
        let eq_handle=eq.clone();
        move |gain_db: f32|{
            ui_handle.global::<Eq>().set_gain_db(gain_db);
            eq_handle.write().unwrap().gain=eq::Gain::Db(gain_db as f64);
        }
    });

    ui_eq.set_min_frequency(app::MIN_FREQUENCY as f32);
    ui_eq.set_max_frequency(app::MAX_FREQUENCY as f32);
    ui_eq.set_min_log_frequency(app::MIN_LOG_FREQUENCY as f32);
    ui_eq.set_max_log_frequency(app::MAX_LOG_FREQUENCY as f32);
    ui_eq.set_frequency(eq.read().unwrap().frequency.hz() as f32);
    ui_eq.set_log_frequency(eq.read().unwrap().frequency.log_hz() as f32);
    ui.global::<Callbacks>().on_request_set_log_frequency({
        let ui_handle = ui.as_weak().unwrap();
        let eq_handle=eq.clone();
        move |log_frequency: f32|{
            let frequency=eq::Frequency::LogHz(log_frequency as f64);
            let ui_eq=ui_handle.global::<Eq>();
            ui_eq.set_log_frequency(frequency.hz() as f32);
            ui_eq.set_frequency(frequency.hz() as f32);
            eq_handle.write().unwrap().frequency=frequency;
        }
    });

    ui_eq.set_min_q(app::MIN_Q as f32);
    ui_eq.set_max_q(app::MAX_Q as f32);
    ui_eq.set_q(eq.read().unwrap().q as f32);
    ui.global::<Callbacks>().on_request_set_q({
        let ui_handle = ui.as_weak().unwrap();
        let eq_handle=eq.clone();
        move |q: f32|{
            ui_handle.global::<Eq>().set_q(q);
            eq_handle.write().unwrap().q=q as f64;
        }
    });


    ui.run()?;

    Ok(())
}
