use egui::{Align, Layout};

pub enum Message {
    FilesLoaded(Vec<Vec<u8>>),
    // Other messages
}

pub struct Gui {
    pub files_to_load: Vec<Vec<u8>>,

    pub ses_resolution: u32,
    pub probe_radius: f32,

    pub render_spacefill: bool,
    pub render_ses_surface: bool,

    pub compute_ses: bool,
    pub compute_ses_once: bool,
    pub frame_time: f32,

    pub error: Option<String>,

    message_channel: (
        std::sync::mpsc::Sender<Message>,
        std::sync::mpsc::Receiver<Message>,
    ),
}

impl Default for Gui {
    fn default() -> Self {
        Self {
            files_to_load: Vec::new(),
            ses_resolution: 64,
            probe_radius: 1.4,
            render_spacefill: true,
            render_ses_surface: true,

            frame_time: 0.0,
            compute_ses: false,
            compute_ses_once: true,
            error: None,

            message_channel: std::sync::mpsc::channel(),
        }
    }
}

impl Gui {
    pub fn ui(&mut self, ctx: &egui::Context, config: &wgpu::SurfaceConfiguration) {
        while let Ok(message) = self.message_channel.1.try_recv() {
            match message {
                Message::FilesLoaded(data) => {
                    self.files_to_load.extend(data);
                }
            }
        }

        egui::containers::Window::new("Settings")
            .default_pos(egui::Pos2::new(100.0, 100.0))
            .show(ctx, |ui| {
                if ui
                    .add(
                        egui::Slider::new(&mut self.ses_resolution, 10..=160)
                            .text("SES resolution"),
                    )
                    .changed()
                {
                    self.compute_ses_once = true;
                };

                ui.add(egui::Slider::new(&mut self.probe_radius, 1.0..=5.0).text("Probe radius"));
                ui.add(egui::Checkbox::new(
                    &mut self.render_spacefill,
                    "Render Spacefill",
                ));
                ui.add(egui::Checkbox::new(
                    &mut self.render_ses_surface,
                    "Render SES surface",
                ));
                ui.separator();
                ui.add(egui::Checkbox::new(
                    &mut self.compute_ses,
                    "Compute SES continuously",
                ));
                // Add fps text
                ui.label(format!("Frame time: {:.3} ms", self.frame_time));

                ui.label(format!("FPS: {}", (1.0 / self.frame_time) as u32));

                ui.label(format!("config: {:?} {:?}", config.width, config.height));
            });

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Organize windows").clicked() {
                        ui.ctx().memory().reset_areas();
                        ui.close_menu();
                    }
                    if ui.button("Load file").clicked() {
                        let task = rfd::AsyncFileDialog::new().pick_files();

                        let message_sender = self.message_channel.0.clone();

                        execute(async move {
                            if let Some(files) = task.await {
                                let mut contents = Vec::new();
                                for file in files {
                                    contents.push(file.read().await);
                                }
                                message_sender.send(Message::FilesLoaded(contents)).ok();
                            }
                        })
                    }
                });
            });
        });

        // Cannot borrow twice as mutable.
        let mut close_error = false;

        if let Some(error_message) = &self.error {
            egui::containers::Window::new("Error")
                .collapsible(false)
                .default_pos((256.0, 256.0))
                .show(ctx, |ui| {
                    ui.with_layout(Layout::top_down(Align::Center), |ui| {
                        ui.label(error_message);
                        if ui.button("Close").clicked() {
                            close_error = true;
                        }
                    });
                });
        }
        if close_error {
            self.error = None;
        }
    }
}

use std::future::Future;

#[cfg(not(target_arch = "wasm32"))]
fn execute<F: Future<Output = ()> + Send + 'static>(f: F) {
    // this is stupid... use any executor of your choice instead
    std::thread::spawn(move || futures::executor::block_on(f));
}
#[cfg(target_arch = "wasm32")]
fn execute<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}
