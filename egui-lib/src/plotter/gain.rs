use crate::*;
use audio_lib::utils as audio_utils;

pub struct EqDiff<F: audio_utils::Float> {
    pub log_frequency: F,
    pub gain_db: F,
}

pub struct IndexedEqDiff<F: audio_utils::Float> {
    pub index: usize,
    pub diff: Option<EqDiff<F>>,
}

#[cfg(not(feature = "analyzer_data"))]
pub fn add_plot<
    F: audio_utils::Float + egui::emath::Numeric,
    const NUM_SPECTRUM_BINS: usize,
    const NUM_SPECTRUM_CHANNELS: usize,
>(
    ui: &mut egui::Ui,
    coefficients: &[Option<biquad::coefficients::Coefficients<F>>],
    sample_rate: F,
    last_eq_index: usize,
    eq_ranges: &EqRanges<F>,
    plot_size: f32,
    color_palette: &colors::ColorPalette,
) -> IndexedEqDiff<F> {
    add_plot_impl::<F, 0, 0>(
        ui,
        coefficients,
        sample_rate,
        last_eq_index,
        eq_ranges,
        plot_size,
        color_palette,
    )
}

#[cfg(feature = "analyzer_data")]
pub fn add_plot<
    F: audio_utils::Float + egui::emath::Numeric,
    const NUM_SPECTRUM_BINS: usize,
    const NUM_SPECTRUM_CHANNELS: usize,
>(
    ui: &mut egui::Ui,
    coefficients: &[Option<biquad::coefficients::Coefficients<F>>],
    sample_rate: F,
    last_eq_index: usize,
    eq_ranges: &EqRanges<F>,
    spectrum_data: &plotter::SpectrumData<F, NUM_SPECTRUM_BINS, NUM_SPECTRUM_CHANNELS>,
    plot_size: f32,
    color_palette: &colors::ColorPalette,
) -> IndexedEqDiff<F> {
    add_plot_impl(
        ui,
        coefficients,
        sample_rate,
        last_eq_index,
        eq_ranges,
        spectrum_data,
        plot_size,
        color_palette,
    )
}

fn add_plot_impl<
    F: audio_utils::Float + egui::emath::Numeric,
    const NUM_SPECTRUM_BINS: usize,
    const NUM_SPECTRUM_CHANNELS: usize,
>(
    ui: &mut egui::Ui,
    coefficients: &[Option<biquad::coefficients::Coefficients<F>>],
    sample_rate: F,
    last_eq_index: usize,
    eq_ranges: &EqRanges<F>,
    #[cfg(feature = "analyzer_data")] spectrum_data: &plotter::SpectrumData<
        F,
        NUM_SPECTRUM_BINS,
        NUM_SPECTRUM_CHANNELS,
    >,
    plot_size: f32,
    color_palette: &colors::ColorPalette,
) -> IndexedEqDiff<F> {
    let gain_plot_id = ui.make_persistent_id("gain_plot_id");
    let plot = egui_plot::Plot::new("Gain (dB)")
        .id(gain_plot_id)
        .allow_zoom(false)
        .allow_drag(false)
        .allow_scroll(false)
        .width(plot_size)
        .height(plot_size)
        .auto_bounds([false, false])
        .custom_x_axes(vec![
            egui_plot::AxisHints::new_x()
                .label("Frequency (Hz)")
                .formatter(|_, _| String::new())
                .placement(egui_plot::VPlacement::Top),
            egui_plot::AxisHints::new_x().formatter(utils::log_frequency_formatter),
        ])
        .custom_y_axes(vec![
            egui_plot::AxisHints::new_y()
                .label("Gain (dB)")
                .formatter(|_, _| String::new()),
            egui_plot::AxisHints::new_y().placement(egui_plot::HPlacement::Right),
        ])
        .label_formatter(|_, point| {
            format!(
                "{} Hz, {:.2} dB",
                audio_utils::log_to_frequency(point.x) as i32,
                point.y
            )
        })
        .legend(egui_plot::Legend::default());

    let eq_ids = (0..coefficients.len())
        .map(|i| ui.make_persistent_id(format!("eq_id_{}", i)))
        .collect::<Vec<egui::Id>>();
    let eq_id_to_index = |id: egui::Id| eq_ids.iter().position(|eq_id| eq_id.value() == id.value());

    let log_frequency_range = &eq_ranges.log_frequency_range;
    let db_range = &eq_ranges.db_range;

    let plot_response = plot.show(ui, |plot_ui| {
        plot_ui.set_plot_bounds(egui_plot::PlotBounds::from_min_max(
            [
                log_frequency_range.start().to_f64().unwrap(),
                db_range.start().to_f64().unwrap(),
            ],
            [
                log_frequency_range.end().to_f64().unwrap(),
                db_range.end().to_f64().unwrap(),
            ],
        ));

        #[cfg(feature = "analyzer_data")]
        {
            let spectrum_rectangles =
                make_spectrum_rectangles(spectrum_data, &log_frequency_range, &db_range);
            for rectangle in spectrum_rectangles {
                let plot_points = egui_plot::PlotPoints::new(rectangle);
                plot_ui.polygon(
                    egui_plot::Polygon::new("", plot_points)
                        .width(1_f32)
                        .fill_color(color_palette.spectrum_fill)
                        .stroke(egui::Stroke::new(1_f32, color_palette.spectrum_fill)),
                );
            }
        }

        let active_coefficients = coefficients.iter().filter(|c| c.is_some());
        if active_coefficients.clone().take(2).count() > 1 {
            let multiband_frequency_response = biquad::utils::multiband::make_frequency_response(
                active_coefficients.map(|c| c.as_ref().unwrap().clone()),
                sample_rate,
            );
            let gain_points = utils::make_log_frequency_points(
                audio_utils::make_gain_db_response(multiband_frequency_response),
                log_frequency_range,
            );
            plot_ui.line(
                egui_plot::Line::new("multiband", gain_points)
                    .color(color_palette.multiband_stroke),
            );
        }
        for (index, c) in coefficients.iter().enumerate() {
            if let Some(c) = c {
                let response = biquad::utils::make_frequency_response(c.clone(), sample_rate);
                let gain_points = utils::make_log_frequency_points(
                    audio_utils::make_gain_db_response(response),
                    log_frequency_range,
                );
                let eq_id = eq_ids[index];
                plot_ui.line(
                    egui_plot::Line::new("", gain_points)
                        .id(eq_id)
                        .color(color_palette.eq_stroke[index % color_palette.eq_stroke.len()]),
                );
            }
        }

        plot_ui.pointer_coordinate_drag_delta()
    });

    let mut selected_eq_index = last_eq_index;
    if plot_response.response.is_pointer_button_down_on() {
        if selected_eq_index >= coefficients.len()
            && let Some(hovered_item) = plot_response.hovered_plot_item
        {
            if let Some(hovered_eq_index) = eq_id_to_index(hovered_item) {
                selected_eq_index = hovered_eq_index;
            }
        }
    } else {
        selected_eq_index = usize::MAX;
    }

    if selected_eq_index < coefficients.len() {
        let drag_delta = plot_response.inner;
        IndexedEqDiff {
            index: selected_eq_index,
            diff: Some(EqDiff {
                log_frequency: F::from(drag_delta.x).unwrap(),
                gain_db: F::from(drag_delta.y).unwrap(),
            }),
        }
    } else {
        IndexedEqDiff {
            index: selected_eq_index,
            diff: None,
        }
    }
}

#[cfg(feature = "analyzer_data")]
fn make_spectrum_rectangles<
    F: audio_utils::Float + egui::emath::Numeric,
    const NUM_SPECTRUM_BINS: usize,
    const NUM_SPECTRUM_CHANNELS: usize,
>(
    spectrum_data: &plotter::SpectrumData<F, NUM_SPECTRUM_BINS, NUM_SPECTRUM_CHANNELS>,
    log_frequency_range: &std::ops::RangeInclusive<F>,
    db_range: &std::ops::RangeInclusive<F>,
) -> Vec<Vec<[f64; 2]>> {
    let bins = &spectrum_data.frequency_bins.bins();
    let mut rectangles: Vec<Vec<[f64; 2]>> =
        Vec::with_capacity(NUM_SPECTRUM_BINS * NUM_SPECTRUM_CHANNELS);

    let bin_rectangle = |bin: &fft::LogFrequencyRangeBin<F>, gain_db: F| -> Option<Vec<[f64; 2]>> {
        let min_x = log_frequency_range
            .start()
            .max(*bin.log_frequency_range.start())
            .to_f64();
        let max_x = log_frequency_range
            .end()
            .min(*bin.log_frequency_range.end())
            .to_f64();
        if min_x > max_x {
            return None;
        }
        let min_y = db_range.start().to_f64().unwrap();
        let max_y = gain_db.clamp(*db_range.start(), *db_range.end()).to_f64();
        if min_y >= max_y {
            return None;
        }
        Some(vec![
            [max_x, min_y],
            [max_x, max_y],
            [min_x, max_y],
            [min_x, min_y],
        ])
    };

    for channel_gains in spectrum_data.linear_gains.iter() {
        for i in 0..NUM_SPECTRUM_BINS {
            if let Some(rect) =
                bin_rectangle(&bins[i], audio_utils::amplitude_to_db(channel_gains[i]))
            {
                rectangles.push(rect);
            }
        }
    }

    rectangles
}
