use plotters::prelude::*;

pub struct ChartColors {
    pub background: RGBColor,
    pub text: RGBAColor,
    pub line: RGBAColor,
    pub plot: RGBColor,
}

pub struct ChartFonts {
    pub caption: FontDesc<'static>,
    pub label: FontDesc<'static>,
}

pub struct ChartStyleData {
    pub colors: ChartColors,
    pub fonts: ChartFonts,
    pub margin_size: u32,
    pub label_area_size: u32,
}

impl ChartStyleData {
    pub fn new(background_color: &slint::Color, area_height: f64) -> Self {
        let label_size = 10.0f64.max(area_height / 35.0);
        ChartStyleData {
            colors: ChartColors {
                background: RGBColor(
                    background_color.red(),
                    background_color.green(),
                    background_color.blue(),
                ),
                text: RGBColor(
                    255 - background_color.red(),
                    255 - background_color.green(),
                    255 - background_color.blue(),
                )
                .mix(0.8),
                line: RGBColor(
                    255 - background_color.red(),
                    255 - background_color.green(),
                    255 - background_color.blue(),
                )
                .mix(0.05),
                plot: RGBColor(255, 100, 0),
            },
            fonts: ChartFonts {
                caption: FontDesc::new(FontFamily::SansSerif, 1.2 * label_size, FontStyle::Bold),
                label: FontDesc::new(FontFamily::SansSerif, label_size, FontStyle::Normal),
            },
            margin_size: 2 * label_size.round() as u32,
            label_area_size: (1.5 * label_size).round() as u32,
        }
    }

    pub fn caption_text_style<'a>(&self) -> TextStyle<'a> {
        self.fonts.caption.color(&self.colors.text)
    }

    pub fn label_text_style<'a>(&self) -> TextStyle<'a> {
        self.fonts.label.color(&self.colors.text)
    }
}
