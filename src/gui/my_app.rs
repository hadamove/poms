#![allow(dead_code)]

pub struct MyApp {
    pub file_to_load: Option<String>,

    pub ses_resolution: u32,
    pub render_spacefill: bool,
    pub render_ses_surface: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            file_to_load: None,
            ses_resolution: 160,
            render_spacefill: true,
            render_ses_surface: false,
        }
    }
}

impl epi::App for MyApp {
    fn name(&self) -> &str {
        "Molecular Surface Explorer"
    }

    fn update(&mut self, ctx: &epi::egui::Context, _frame: &epi::Frame) {
        egui::containers::Window::new("Settings").show(ctx, |ui| {
            ui.add(egui::Slider::new(&mut self.ses_resolution, 10..=160).text("SES resolution"));
            ui.add(egui::Checkbox::new(
                &mut self.render_spacefill,
                "Render Spacefill",
            ));
            ui.add(egui::Checkbox::new(
                &mut self.render_ses_surface,
                "Render SES surface",
            ));
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
                    if ui.button("Open").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            self.file_to_load = Some(path.display().to_string());
                            dbg!(&self.file_to_load);
                        }
                    }
                });
            });
        });
    }
}
