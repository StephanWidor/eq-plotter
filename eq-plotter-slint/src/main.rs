// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod eq_plotter;
pub mod plotters;

pub type EqRanges = app_lib::settings::EqRanges<f32>;
pub type ImpulseResponseSettings = app_lib::settings::ImpulseResponse<f32>;
pub type Settings = app_lib::settings::Settings<f32, 1>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let eq_plotter = eq_plotter::EqPlotter::new(&Settings::default())?;
    eq_plotter.run()?;
    Ok(())
}
