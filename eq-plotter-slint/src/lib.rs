// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod eq_plotter;
pub mod plotters;
