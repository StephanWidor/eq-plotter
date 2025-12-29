use app_lib as app;
use audio_lib::biquad;
use audio_lib::eq;
use audio_lib::utils;
use num::complex::ComplexFloat;
use plotters::prelude::*;
use slint::SharedPixelBuffer;

pub fn render_eq_plots(
    eq: &eq::Eq<f32>,
    sample_rate: f32,
    width: u32,
    height: u32,
    background_color: slint::Color,
) -> slint::Image {
    let mut pixel_buffer = SharedPixelBuffer::new(width, height);
    let backend = BitMapBackend::with_buffer(pixel_buffer.make_mut_bytes(), (width, height));
    let root_area = backend.into_drawing_area();

    let chart_style = ChartStyleData::new(&background_color, height as f64 / 2.0);

    root_area
        .fill(&chart_style.colors.background)
        .expect("error filling drawing area");
    let plot_areas = root_area.split_evenly((2, 2));

    let coefficients = biquad::coefficients::Coefficients::from_eq(eq, sample_rate);
    let frequency_response =
        biquad::utils::make_frequency_response_function(&coefficients, sample_rate);

    draw_gain_chart(&plot_areas[0], &chart_style, &frequency_response);
    draw_phase_chart(&plot_areas[2], &chart_style, &frequency_response);
    draw_ir_chart(&plot_areas[1], &chart_style, &coefficients);
    draw_poles_and_zeros_chart(&plot_areas[3], &chart_style, &coefficients);

    root_area.present().expect("error presenting");

    for plot_area in plot_areas {
        drop(plot_area);
    }
    drop(root_area);

    slint::Image::from_rgb8(pixel_buffer)
}

struct ChartColors {
    pub background: RGBColor,
    pub text: RGBAColor,
    pub line: RGBAColor,
    pub plot: RGBColor,
}

struct ChartFonts {
    pub caption: FontDesc<'static>,
    pub label: FontDesc<'static>,
}

struct ChartStyleData {
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

fn draw_gain_chart<DB: DrawingBackend>(
    area: &DrawingArea<DB, plotters::coord::Shift>,
    style: &ChartStyleData,
    frequency_response: &impl Fn(f32) -> num::Complex<f32>,
) {
    let log_frequency_steps =
        (app::MIN_LOG_FREQUENCY as f32..app::MAX_LOG_FREQUENCY as f32).step(0.01);
    let gain_db_min = app::MIN_GAIN_DB as f32;
    let gain_db_max = app::MAX_GAIN_DB as f32;

    let mut chart = ChartBuilder::on(area)
        .margin(style.margin_size)
        .set_all_label_area_size(style.label_area_size)
        .caption("Gain Response", style.caption_text_style())
        .build_cartesian_2d(log_frequency_steps.range(), gain_db_min..gain_db_max)
        .expect("error creating gain response chart")
        .set_secondary_coord(0f32..1f32, 0f32..1f32);

    chart
        .configure_mesh()
        .x_labels(10)
        .y_labels(10)
        .axis_style(style.colors.text)
        .bold_line_style(style.colors.line)
        .light_line_style(style.colors.line)
        .label_style(style.label_text_style())
        .x_label_formatter(&|log_frequency| {
            format!("{:.0}", utils::log_to_frequency(*log_frequency))
        })
        .y_label_formatter(&|gain_db| format!("{:.0}", gain_db))
        .draw()
        .expect("error drawing gain response mesh");

    chart
        .configure_secondary_axes()
        .x_desc("Frquency (Hz)")
        .y_desc("Gain (dB)")
        .x_labels(0)
        .y_labels(0)
        .axis_style(style.colors.line)
        .label_style(style.label_text_style())
        .draw()
        .expect("error drawing gain response secondary axes");

    let map_gains = |log_frequency| {
        (
            log_frequency,
            utils::amplitude_to_db(
                frequency_response(utils::log_to_frequency(log_frequency)).abs(),
            ),
        )
    };
    chart
        .draw_series(LineSeries::new(
            log_frequency_steps.values().map(map_gains),
            &style.colors.plot,
        ))
        .expect("error drawing gain points");
}

fn draw_phase_chart<DB: DrawingBackend>(
    area: &DrawingArea<DB, plotters::coord::Shift>,
    style: &ChartStyleData,
    frequency_response: &impl Fn(f32) -> num::Complex<f32>,
) {
    let log_frequency_steps =
        (app::MIN_LOG_FREQUENCY as f32..app::MAX_LOG_FREQUENCY as f32).step(0.01);
    let phase_min = -std::f32::consts::PI;
    let phase_max = std::f32::consts::PI;

    let mut chart = ChartBuilder::on(&area)
        .margin(style.margin_size)
        .set_all_label_area_size(style.label_area_size)
        .caption("Phase Response", style.caption_text_style())
        .build_cartesian_2d(log_frequency_steps.range(), phase_min..phase_max)
        .expect("error creating phase response chart")
        .set_secondary_coord(0f32..1f32, 0f32..1f32);

    chart
        .configure_mesh()
        .x_labels(10)
        .y_labels(10)
        .axis_style(style.colors.text)
        .bold_line_style(style.colors.line)
        .light_line_style(style.colors.line)
        .label_style(style.label_text_style())
        .x_label_formatter(&|log_frequency| {
            format!("{:.0}", utils::log_to_frequency(*log_frequency))
        })
        .y_label_formatter(&|phase| format!("{:.2}", phase))
        .draw()
        .expect("error drawing phase response mesh");

    chart
        .configure_secondary_axes()
        .x_desc("Frquency (Hz)")
        .y_desc("Phase (rad)")
        .x_labels(0)
        .y_labels(0)
        .axis_style(style.colors.line)
        .label_style(style.label_text_style())
        .draw()
        .expect("error drawing phase response secondary axes");

    let map_phases = |log_frequency: f32| {
        (
            log_frequency,
            frequency_response(utils::log_to_frequency(log_frequency)).arg(),
        )
    };
    chart
        .draw_series(LineSeries::new(
            log_frequency_steps.values().map(map_phases),
            &style.colors.plot,
        ))
        .expect("error drawing phase points");
}

fn draw_ir_chart<DB: DrawingBackend>(
    area: &DrawingArea<DB, plotters::coord::Shift>,
    style: &ChartStyleData,
    coefficients: &biquad::coefficients::Coefficients<f32>,
) {
    let impulse_response = biquad::utils::impulse_response(&coefficients, 0.001, 10, 1024);

    let mut chart = ChartBuilder::on(&area)
        .margin(style.margin_size)
        .set_all_label_area_size(style.label_area_size)
        .caption("Impulse Response", style.caption_text_style())
        .build_cartesian_2d(0f32..(impulse_response.len() as f32), -1f32..1f32)
        .expect("error creating impulse response chart")
        .set_secondary_coord(0f32..1f32, 0f32..1f32);

    chart
        .configure_mesh()
        .x_labels(10)
        .y_labels(10)
        .axis_style(style.colors.text)
        .bold_line_style(style.colors.line)
        .light_line_style(style.colors.line)
        .label_style(style.label_text_style())
        .x_label_formatter(&|i| format!("{}", i))
        .y_label_formatter(&|r| format!("{:.2}", r))
        .draw()
        .expect("error drawing impulse response mesh");

    chart
        .configure_secondary_axes()
        .x_labels(0)
        .y_labels(0)
        .axis_style(style.colors.line)
        .draw()
        .expect("error drawing impulse response secondary axes");

    chart
        .draw_series(LineSeries::new(
            (0..impulse_response.len()).map(|i| (i as f32, impulse_response[i])),
            &style.colors.plot,
        ))
        .expect("error drawing impulse response points");
}

fn draw_poles_and_zeros_chart<DB: DrawingBackend>(
    area: &DrawingArea<DB, plotters::coord::Shift>,
    style: &ChartStyleData,
    coefficients: &biquad::coefficients::Coefficients<f32>,
) {
    let mut chart = ChartBuilder::on(&area)
        .margin(style.margin_size)
        .set_all_label_area_size(style.label_area_size)
        .caption("Poles and Zeros", style.caption_text_style())
        .build_cartesian_2d(-1f32..1f32, -1f32..1f32)
        .expect("error creating poles and zeros chart")
        .set_secondary_coord(0f32..1f32, 0f32..1f32);

    chart
        .configure_mesh()
        .x_labels(10)
        .y_labels(10)
        .axis_style(style.colors.text)
        .bold_line_style(style.colors.line)
        .light_line_style(style.colors.line)
        .label_style(style.label_text_style())
        .x_label_formatter(&|x| format!("{:.2}", x))
        .y_label_formatter(&|y| format!("{:.2}", y))
        .draw()
        .expect("error drawing poles and zeros mesh");

    chart
        .configure_secondary_axes()
        .x_labels(0)
        .y_labels(0)
        .axis_style(style.colors.line)
        .draw()
        .expect("error drawing poles and zeros secondary axes");

    let radian_steps = (0f32..2f32 * std::f32::consts::PI).step(0.01);
    chart
        .draw_series(LineSeries::new(
            radian_steps.values().map(|a| (a.cos(), a.sin())),
            &style.colors.plot,
        ))
        .expect("error drawing unit circle");

    let poles = biquad::utils::poles(&coefficients);
    if !poles.is_empty() {
        let poles_series = chart
            .draw_series(PointSeries::of_element(
                poles.iter().map(|pole| (pole.re, pole.im)),
                poles.len() as u32,
                ShapeStyle::from(&RED).filled(),
                &|coord, size, style| EmptyElement::at(coord) + Circle::new((0, 0), size, style),
            ))
            .expect("error drawing poles");
        poles_series
            .label("Poles")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));
    }

    let zeros = biquad::utils::zeros(&coefficients);
    if !zeros.is_empty() {
        let zeros_series = chart
            .draw_series(PointSeries::of_element(
                zeros.iter().map(|zero| (zero.re, zero.im)),
                zeros.len() as u32,
                ShapeStyle::from(&GREEN).filled(),
                &|coord, size, style| EmptyElement::at(coord) + Circle::new((0, 0), size, style),
            ))
            .expect("error drawing zeros");
        zeros_series
            .label("Zeros")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], GREEN));
    }

    chart
        .configure_series_labels()
        .label_font(style.label_text_style())
        .position(SeriesLabelPosition::UpperRight)
        .draw()
        .expect("error drawing poles and zeros series labels");
}
