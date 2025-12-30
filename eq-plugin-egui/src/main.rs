use eq_plugin_egui::plugin;
use nih_plug::prelude::*;

fn main() {
    nih_export_standalone::<plugin::EqPlugin>();
}
