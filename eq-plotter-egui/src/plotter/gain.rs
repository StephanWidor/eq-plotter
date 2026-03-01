use crate::*;
use app_lib as app;
use audio_lib::eq;
use audio_lib::fft;
use audio_lib::utils as audio_utils;
use egui::emath::Numeric;

pub struct SpectrumData<'a, const NUM_BINS: usize, const NUM_CHANNELS: usize> {
    pub frequency_bins: &'a fft::LogFrequencyBins<f32, NUM_BINS>,
    pub linear_gains: &'a [[f32; NUM_BINS]; NUM_CHANNELS],
}

pub fn add_plot<const NUM_SPECTRUM_BINS: usize, const NUM_SPECTRUM_CHANNELS: usize>(
    ui: &mut egui::Ui,
    eqs: &mut [eq::Eq<f64>],
    selected_eq_index: &mut usize,
    spectrum_data: &Option<SpectrumData<NUM_SPECTRUM_BINS, NUM_SPECTRUM_CHANNELS>>,
    frequency_responses: &[impl Fn(f64) -> num::Complex<f64>],
    multiband_frequency_response: &impl Fn(f64) -> num::Complex<f64>,
    plot_size: f32,
) {
    assert!(eqs.len() == frequency_responses.len());
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

    let eq_ids = (0..eqs.len())
        .map(|i| ui.make_persistent_id(format!("eq_id_{}", i)))
        .collect::<Vec<egui::Id>>();
    let eq_id_to_index = |id: egui::Id| eq_ids.iter().position(|eq_id| eq_id.value() == id.value());

    let plot_response = plot.show(ui, |plot_ui| {
        plot_ui.set_plot_bounds(egui_plot::PlotBounds::from_min_max(
            [app::MIN_LOG_FREQUENCY, app::MIN_GAIN_DB],
            [app::MAX_LOG_FREQUENCY, app::MAX_GAIN_DB],
        ));

        if let Some(spectrum_data) = spectrum_data {
            let spectrum_rectangles = make_spectrum_rectangles(
                spectrum_data,
                app::MIN_LOG_FREQUENCY..=app::MAX_LOG_FREQUENCY,
                app::MIN_GAIN_DB..=app::MAX_GAIN_DB,
            );
            for rectangle in spectrum_rectangles.iter() {
                let plot_points = egui_plot::PlotPoints::new(rectangle.clone()); // TODO: use Borrowed
                plot_ui.polygon(
                    egui_plot::Polygon::new("", plot_points)
                        .width(1_f32)
                        .fill_color(constants::SPECTRUM_COLOR)
                        .stroke(egui::Stroke::new(1_f32, constants::SPECTRUM_COLOR)),
                );
            }
        }

        let mut num_active = 0;
        for index in 0..eqs.len() {
            let eq = &eqs[index];
            let eq_id = eq_ids[index];
            let response = &frequency_responses[index];
            if eq.eq_type == eq::EqType::Bypassed {
                continue;
            }
            num_active += 1;
            let gain_points =
                utils::make_log_frequency_points(audio_utils::make_gain_db_response(response));
            plot_ui.line(
                egui_plot::Line::new("", gain_points)
                    .id(eq_id)
                    .color(constants::EQ_COLORS[index % constants::EQ_COLORS.len()]),
            );
        }
        if num_active > 1 {
            let gain_points = utils::make_log_frequency_points(audio_utils::make_gain_db_response(
                multiband_frequency_response,
            ));
            plot_ui.line(
                egui_plot::Line::new("multiband", gain_points)
                    .fill_alpha(0.5)
                    .color(constants::MULTI_BAND_COLOR),
            );
        }
        plot_ui.pointer_coordinate_drag_delta()
    });

    if plot_response.response.is_pointer_button_down_on() {
        if *selected_eq_index >= eqs.len()
            && let Some(hovered_item) = plot_response.hovered_plot_item
        {
            if let Some(hovered_eq_index) = eq_id_to_index(hovered_item) {
                *selected_eq_index = hovered_eq_index;
            }
        }
    } else {
        *selected_eq_index = usize::MAX;
    }

    if *selected_eq_index < eqs.len() {
        let drag_delta = plot_response.inner;
        let eq = &mut eqs[*selected_eq_index];
        eq.frequency = eq::Frequency::LogHz(eq.frequency.log_hz() + drag_delta.x as f64);
        eq.gain = eq::Gain::Db(eq.gain.db() + drag_delta.y as f64);
    }
}

fn make_spectrum_rectangles<const NUM_SPECTRUM_BINS: usize, const NUM_SPECTRUM_CHANNELS: usize>(
    spectrum_data: &SpectrumData<NUM_SPECTRUM_BINS, NUM_SPECTRUM_CHANNELS>,
    log_frequency_range: std::ops::RangeInclusive<f64>,
    db_range: std::ops::RangeInclusive<f64>,
) -> Vec<Vec<[f64; 2]>> {
    let bins = &spectrum_data.frequency_bins.bins();
    let mut rectangles: Vec<Vec<[f64; 2]>> =
        Vec::with_capacity(NUM_SPECTRUM_BINS * NUM_SPECTRUM_CHANNELS);

    let bin_rectangle = |bin: &fft::LogFrequencyBin<f32>, gain_db: f32| -> Option<Vec<[f64; 2]>> {
        let min_x = log_frequency_range
            .start()
            .max(*bin.frequency_range.start() as f64);
        let max_x = log_frequency_range
            .end()
            .min(*bin.frequency_range.end() as f64);
        if min_x > max_x {
            return None;
        }
        let min_y = *db_range.start();
        let max_y = gain_db.to_f64().clamp(*db_range.start(), *db_range.end());
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
