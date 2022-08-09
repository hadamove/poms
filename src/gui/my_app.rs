pub struct MyApp {
    pub colors: ColorWidgets,
    pub file_to_load: Option<String>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            colors: Default::default(),
            file_to_load: None,
        }
    }
}

impl epi::App for MyApp {
    fn name(&self) -> &str {
        "Atom colors"
    }

    fn update(&mut self, ctx: &epi::egui::Context, _frame: &epi::Frame) {
        // egui::containers::Window::new(self.name())
        //     .vscroll(true)
        //     .hscroll(true)
        //     .show(ctx, |ui| {
        //         self.colors.ui(ui);
        //     });

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
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

#[derive(PartialEq)]
pub struct ColorWidgets {
    pub h_color: [f32; 4],
    pub c_color: [f32; 4],
    pub o_color: [f32; 4],
    pub n_color: [f32; 4],
    pub s_color: [f32; 4],
}

impl Default for ColorWidgets {
    fn default() -> Self {
        ColorWidgets {
            h_color: [1.0, 1.0, 1.0, 1.0],
            c_color: [0.2, 0.2, 0.2, 1.0],
            o_color: [1.0, 0.0, 0.0, 1.0],
            n_color: [0.0, 0.0, 1.0, 1.0],
            s_color: [1.0, 1.0, 0.0, 1.0],
        }
    }
}

impl ColorWidgets {
    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::reset_button(ui, self);

        let Self {
            h_color,
            c_color,
            o_color,
            n_color,
            s_color,
        } = self;

        ui.horizontal(|ui| {
            ui.color_edit_button_rgba_unmultiplied(h_color);
            ui.label(format!("Hydrogen",));
        });

        ui.horizontal(|ui| {
            ui.color_edit_button_rgba_unmultiplied(c_color);
            ui.label(format!("Carbon",));
        });

        ui.horizontal(|ui| {
            ui.color_edit_button_rgba_unmultiplied(o_color);
            ui.label(format!("Oxygen"));
        });

        ui.horizontal(|ui| {
            ui.color_edit_button_rgba_unmultiplied(n_color);
            ui.label(format!("Nitrogen"));
        });

        ui.horizontal(|ui| {
            ui.color_edit_button_rgba_unmultiplied(s_color);
            ui.label(format!("Sulfur"));
        });
    }
}
