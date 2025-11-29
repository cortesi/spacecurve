//! Web entrypoint and glue for the spacecurve GUI compiled to WebAssembly.
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_sys::{HtmlCanvasElement, UrlSearchParams, window};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
/// Launch the spacecurve GUI inside the existing canvas element when running on the web.
pub async fn run() {
    console_error_panic_hook::set_once();

    let dev_mode = query_flag("dev");
    let include_experimental = dev_mode || query_flag("experimental");

    let web_options = eframe::WebOptions::default();
    // Obtain the canvas element by id from the DOM.
    let document = window()
        .and_then(|w| w.document())
        .expect("document not available");
    let canvas = document
        .get_element_by_id("bevy")
        .expect("canvas with id 'bevy' not found")
        .dyn_into::<HtmlCanvasElement>()
        .expect("element is not a canvas");

    let gui_options = scurve_gui::GuiOptions {
        include_experimental_curves: include_experimental,
        show_dev_overlay: dev_mode,
        ..Default::default()
    };

    eframe::WebRunner::new()
        .start(
            canvas,
            web_options,
            Box::new(move |cc| {
                Ok(Box::new(scurve_gui::ScurveApp::with_options(
                    cc,
                    gui_options.clone(),
                )))
            }),
        )
        .await
        .expect("failed to start eframe web app");
}

#[cfg(target_arch = "wasm32")]
fn query_flag(param: &str) -> bool {
    let search = window()
        .map(|w| w.location())
        .and_then(|loc| loc.search().ok())
        .unwrap_or_default();

    if search.is_empty() {
        return false;
    }

    let Ok(params) = UrlSearchParams::new_with_str(&search) else {
        return false;
    };

    if !params.has(param) {
        return false;
    }

    match params.get(param).as_deref() {
        None => true,
        Some(v) if v.is_empty() => true,
        Some(v) => matches!(
            v.to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "y" | "on"
        ),
    }
}

// Provide a no-op main for non-wasm targets so the bin compiles in workspace builds
#[cfg(not(target_arch = "wasm32"))]
fn main() {}

// Provide an empty main for wasm so binaries satisfy the `main` requirement.
#[cfg(target_arch = "wasm32")]
fn main() {}
