#[cfg(target_arch = "wasm32")]
use crate::app::App;

#[cfg(target_arch = "wasm32")]
use winit::window::Window;

#[cfg(target_arch = "wasm32")]
pub fn init_browser_window(window: &Window) {
    // Log detailed error info to browser's dev console
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    wasm_logger::init(wasm_logger::Config::default());

    // Append window to document body
    use winit::platform::web::WindowExtWebSys;
    web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| doc.body())
        .and_then(|body| {
            let canvas = window.canvas();
            let style = canvas.style();
            // Set canvas to fill the whole window
            style.set_property("width", "100%").unwrap();
            style.set_property("height", "100%").unwrap();
            body.append_child(&web_sys::Element::from(canvas)).ok()
        })
        .expect("Failed to append canvas to body");
}

#[cfg(target_arch = "wasm32")]
pub fn resize_app_if_canvas_changed(window: &Window, app: &mut App) {
    use winit::platform::web::WindowExtWebSys;

    let canvas = window.canvas();
    let (width, height) = (canvas.client_width(), canvas.client_height());

    let logical_size = winit::dpi::LogicalSize { width, height };
    let scale_factor = window.scale_factor();
    let canvas_size = logical_size.to_physical(scale_factor);

    if canvas_size != window.inner_size() {
        canvas.set_width(canvas_size.width);
        canvas.set_height(canvas_size.height);
        app.resize(canvas_size);
    }
}
