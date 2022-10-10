use std::path::PathBuf;

pub struct MyApp {
    pub to_load: Option<ResourcePath>,

    pub ses_resolution: u32,
    pub render_spacefill: bool,
    pub render_ses_surface: bool,

    pub show_distance_field: bool,
    pub df_visualize_layer: u32,

    pub compute_ses: bool,
    pub compute_ses_once: bool,
    pub frame_time: f32,
}

pub enum ResourcePath {
    SingleMolecule(PathBuf),
    DynamicMolecule(PathBuf),
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            to_load: None,
            ses_resolution: 32,
            render_spacefill: true,
            render_ses_surface: true,
            show_distance_field: false,

            df_visualize_layer: 0,
            frame_time: 0.0,
            compute_ses: false,
            compute_ses_once: true,
        }
    }
}

impl MyApp {
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
                    &mut self.show_distance_field,
                    "Show Distance Field",
                ));
                ui.add(
                    egui::Slider::new(&mut self.df_visualize_layer, 0..=self.ses_resolution - 1)
                        .text("Distance Field Layer"),
                );
                ui.separator();
                ui.add(egui::Checkbox::new(
                    &mut self.compute_ses,
                    "Compute SES continuously",
                ));
                // Add fps text
                ui.label(format!("Frame time: {:.3} ms", self.frame_time));
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
                            self.to_load = Some(ResourcePath::SingleMolecule(path))
                        }
                    }
                    if ui.button("Load folder (multiple)").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.to_load = Some(ResourcePath::DynamicMolecule(path))
                        }
                    }
                });
            });
        });
    }
}
