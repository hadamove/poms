use std::path::PathBuf;

use egui::{Align, Layout};

pub struct Gui {
    pub files_to_load: Vec<PathBuf>,

    pub ses_resolution: u32,
    pub render_spacefill: bool,
    pub render_ses_surface: bool,

    pub compute_ses: bool,
    pub compute_ses_once: bool,
    pub frame_time: f32,

    pub error: Option<String>,
}

impl Default for Gui {
    fn default() -> Self {
        Self {
            files_to_load: Vec::new(),
            ses_resolution: 64,
            render_spacefill: true,
            render_ses_surface: true,

            frame_time: 0.0,
            compute_ses: false,
            compute_ses_once: true,
            error: None,
        }
    }
}

impl Gui {
    pub fn ui(&mut self, ctx: &egui::Context) {
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
            });

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // TODO: make this async
                // https://github.com/emilk/egui/issues/270#issuecomment-869069186
                ui.menu_button("File", |ui| {
                    if ui.button("Organize windows").clicked() {
                        ui.ctx().memory().reset_areas();
                        ui.close_menu();
                    }
                    if ui.button("Load file").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            self.files_to_load = vec![path]
                        }
                    }
                    if ui.button("Load folder (multiple)").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            let directory = std::fs::read_dir(path);
                            match directory {
                                Ok(dir) => {
                                    self.files_to_load = dir
                                        .filter_map(|d| d.ok())
                                        .map(|entry| entry.path())
                                        .collect::<Vec<_>>();
                                }
                                Err(e) => {
                                    self.error = Some(format!("Could not open directory: {}", e));
                                }
                            }
                        }
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
