#[derive(Clone)]
pub struct ColorPalette {
    pub background: egui::Color32,
    pub eq_stroke: [egui::Color32; 8],
    pub multiband_stroke: egui::Color32,
    pub spectrum_fill: egui::Color32,
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self {
            background: egui::Color32::from_rgb(32, 35, 38),
            eq_stroke: [
                egui::Color32::from_rgb(140, 51, 51),
                egui::Color32::from_rgb(140, 107, 54),
                egui::Color32::from_rgb(104, 140, 56),
                egui::Color32::from_rgb(59, 140, 106),
                egui::Color32::from_rgb(59, 102, 140),
                egui::Color32::from_rgb(77, 58, 140),
                egui::Color32::from_rgb(140, 60, 140),
                egui::Color32::from_rgb(140, 82, 99),
            ],
            multiband_stroke: egui::Color32::LIGHT_GRAY,
            spectrum_fill: egui::Color32::from_rgb(16, 17, 19),
        }
    }
}
