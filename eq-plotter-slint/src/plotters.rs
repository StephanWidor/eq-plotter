use app_lib as app;
use audio_lib::biquad;
use audio_lib::eq;
use audio_lib::utils;
use num::ToPrimitive;
use num::complex::ComplexFloat;
use num_traits::Pow;
use plotters::prelude::*;
use slint::SharedPixelBuffer;

pub fn render_eq_plots(eq: &eq::Eq<f32>, sample_rate: f32) -> slint::Image {
    let coefficients = biquad::coefficients::Coefficients::from_eq(eq, sample_rate);
    let frequency_response =
        biquad::utils::make_frequency_response_function(&coefficients, sample_rate);

    let num_pixels = 1024;
    let mut pixel_buffer = SharedPixelBuffer::new(num_pixels, num_pixels);
    let pixel_buffer_size = (pixel_buffer.width(), pixel_buffer.height());
    let backend = BitMapBackend::with_buffer(pixel_buffer.make_mut_bytes(), pixel_buffer_size);

    let root_area = backend.into_drawing_area();
    root_area.fill(&RGBColor(app::UI_BACKGROUND_COLOR[0], app::UI_BACKGROUND_COLOR[1], app::UI_BACKGROUND_COLOR[2]))
        .expect("error filling drawing area");

    let half_pixels = num_pixels / 2;
    let (upper_area, lower_area) = root_area.split_vertically(half_pixels);
    let (left_upper_area, right_upper_area) = upper_area.split_horizontally(half_pixels);
    let (left_lower_area, right_lower_area) = lower_area.split_horizontally(half_pixels);
    drop(upper_area);
    drop(lower_area);
    
    let log_frequency_min=app::MIN_LOG_FREQUENCY as f32;
    let log_frequency_max=app::MAX_LOG_FREQUENCY as f32;
    let log_frequency_steps = (log_frequency_min..log_frequency_max).step(0.01);

    let lighter_gray = RGBColor(200, 200, 200);
    let darker_gray = RGBColor(50, 50, 50);
    let plot_color = RGBColor(200, 150, 0);
    let caption_text_style=("sans-serif", 20, FontStyle::Bold, &lighter_gray).into_text_style(&root_area);
    let label_text_style=("sans-serif", 15, FontStyle::Normal, &lighter_gray).into_text_style(&root_area);
    {
        let gain_db_min=app::MIN_GAIN_DB as f32;
        let gain_db_max=app::MAX_GAIN_DB as f32;
        let mut chart = ChartBuilder::on(&left_upper_area)
            .margin(5)
            .set_all_label_area_size(30)
            .top_x_label_area_size(0)
            .caption("Gain Response", caption_text_style.clone())
            .build_cartesian_2d(log_frequency_min..log_frequency_max, gain_db_min..gain_db_max)
            .expect("error creating gain response chart");
        chart.configure_mesh()
            .x_labels(4)
            .x_desc("Frquency (Hz)")
            .y_labels(6)
            .y_desc("Gain (dB)")
            .axis_style(darker_gray)
            .bold_line_style(darker_gray)
            .light_line_style(darker_gray)
            .label_style(label_text_style.clone())
            .x_label_formatter(&|log_frequency| format!("{}", utils::log_to_frequency(*log_frequency)))
            .y_label_formatter(&|gain_db| format!("{:.1}", gain_db))
            .draw()
            .expect("error drawing gain response mesh");
        let map_gains = |log_frequency| {
            (log_frequency, utils::amplitude_to_db(frequency_response(utils::log_to_frequency(log_frequency)).abs()))
            };
        chart.draw_series(LineSeries::new(log_frequency_steps.values().map(map_gains), &plot_color))
            .expect("error drawing gain points");
    }
    {
        let phase_min=-std::f32::consts::PI;
        let phase_max=std::f32::consts::PI;
        let mut chart = ChartBuilder::on(&left_lower_area)
            .margin(5)
            .set_all_label_area_size(30)
            .top_x_label_area_size(0)
            .caption("Phase Response", caption_text_style.clone())
            .build_cartesian_2d(log_frequency_min..log_frequency_max, phase_min..phase_max)
            .expect("error creating phase response chart");
        chart.configure_mesh()
            .x_labels(4)
            .y_labels(6)
            .y_desc("Phase (rad)")
            .axis_style(darker_gray)
            .bold_line_style(darker_gray)
            .light_line_style(darker_gray)
            .label_style(label_text_style.clone())
            .x_label_formatter(&|log_frequency| format!("{}", utils::log_to_frequency(*log_frequency)))
            .y_label_formatter(&|phase| format!("{:.1}", phase))
            .draw()
            .expect("error drawing phase response mesh");
            let map_phases = |log_frequency:f32| {
                (log_frequency, frequency_response(utils::log_to_frequency(log_frequency)).arg())
            };
        chart.draw_series(LineSeries::new(log_frequency_steps.values().map(map_phases), &plot_color))
            .expect("error drawing phase points");
    }
    {
        let impulse_response =
            biquad::utils::impulse_response(&coefficients, 0.001, 10, 1024);

        let mut chart = ChartBuilder::on(&right_upper_area)
            .margin(5)
            .set_all_label_area_size(30)
            .top_x_label_area_size(0)
            .caption("Impulse Response", caption_text_style.clone())
            .build_cartesian_2d(0f32..(impulse_response.len() as f32), -1f32..1f32)
            .expect("error creating impulse response chart");
        chart.configure_mesh()
            .x_labels(4)
            .y_labels(6)
            .axis_style(darker_gray)
            .bold_line_style(darker_gray)
            .light_line_style(darker_gray)
            .label_style(label_text_style.clone())
            .x_label_formatter(&|i| format!("{}", i))
            .y_label_formatter(&|r| format!("{:.1}", r))
            .draw()
            .expect("error drawing impulse response mesh");
        chart.draw_series(
            LineSeries::new((0..impulse_response.len()).map(|i| (i as f32, impulse_response[i])), &plot_color))
            .expect("error drawing impulse response points");
    }
    {
        let mut chart = ChartBuilder::on(&right_lower_area)
            .margin(5)
            .set_all_label_area_size(30)
            .top_x_label_area_size(0)
            .caption("Poles and Zeros", caption_text_style.clone())
            .build_cartesian_2d(-1f32..1f32, -1f32..1f32)
            .expect("error creating poles and zeros chart");
        chart.configure_mesh()
            .x_labels(10)
            .y_labels(10)
            .axis_style(darker_gray)
            .bold_line_style(darker_gray)
            .light_line_style(darker_gray)
            .label_style(label_text_style.clone())
            .x_label_formatter(&|x| format!("{:.1}", x))
            .y_label_formatter(&|y| format!("{:.1}", y))
            .draw()
            .expect("error drawing impulse response mesh");

        let radian_steps = (0f32..2f32*std::f32::consts::PI).step(0.01);
        chart.draw_series(
            LineSeries::new(radian_steps.values().map(|a| (a.cos(), a.sin())), &plot_color))
            .expect("error drawing impulse response points");

        let poles = biquad::utils::poles(&coefficients);
        let poles_series = chart.draw_series(PointSeries::of_element(
            (0..=1).map(|i| (poles[i].re, poles[i].im)),
            2,
            ShapeStyle::from(&RED).filled(),
            &|coord, size, style| {
                EmptyElement::at(coord)
                    + Circle::new((0, 0), size, style)
                },)).expect("error drawing poles");
        poles_series.label("Poles").legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));

        let zeros = biquad::utils::zeros(&coefficients);
        let zeros_series = chart.draw_series(PointSeries::of_element(
            (0..=1).map(|i| (zeros[i].re, zeros[i].im)),
            2,
            ShapeStyle::from(&GREEN).filled(),
            &|coord, size, style| {
                EmptyElement::at(coord)
                    + Circle::new((0, 0), size, style)
                },)).expect("error drawing poles");
        zeros_series.label("Zeros").legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], GREEN));

        chart.configure_series_labels()
            .label_font(label_text_style)
            .position(SeriesLabelPosition::UpperRight)
            .draw()
            .expect("error drawing series labels");

    }
        
    root_area.present().expect("error presenting");
    
    drop(left_upper_area);
    drop(right_upper_area);
    drop(left_lower_area);
    drop(right_lower_area);
    drop(root_area);

    slint::Image::from_rgb8(pixel_buffer)
}
