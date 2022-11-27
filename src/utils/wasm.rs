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
pub fn update_window_size_if_canvas_changed(window: &Window, app: &mut App) {
    use winit::platform::web::WindowExtWebSys;

    let size = window.inner_size();

    let canvas = window.canvas();
    let (width, height) = (canvas.client_width(), canvas.client_height());
    let factor = window.scale_factor();

    let logical = winit::dpi::LogicalSize { width, height };
    let new_size = logical.to_physical(factor);

    if new_size != size {
        canvas.set_width(new_size.width);
        canvas.set_height(new_size.height);
        app.resize(new_size);
    }
}
