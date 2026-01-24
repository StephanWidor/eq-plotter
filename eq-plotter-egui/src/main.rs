#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::wasm_bindgen;

use eq_plotter_egui::*;

#[cfg(target_arch = "wasm32")]
fn main() -> eframe::Result {
    console_error_panic_hook::set_once();
    wasm_bindgen_futures::spawn_local(async {
        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("should have a document on window");
        let canvas = document
            .get_element_by_id("canvas_id")
            .expect("should have a canvas element")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("should be a canvas");

        let web_options = eframe::WebOptions::default();
        let web_runner = eframe::WebRunner::new();
        web_runner
            .start(
                canvas,
                web_options,
                Box::new(|_cc| Ok(Box::new(<EqPlotter>::new(8)))),
            )
            .await
            .expect("failed to start WebRunner");
    });
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([
                constants::INIT_WINDOW_SIZE[0] as f32,
                constants::INIT_WINDOW_SIZE[1] as f32,
            ])
            .with_clamp_size_to_monitor_size(false)
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "EQ Plotter",
        options,
        Box::new(|_cc| Ok(Box::new(eq_plotter::EqPlotter::new(8)))),
    )
}
