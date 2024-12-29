use crate::mod_types::{ModType, Modification, MultiPak, Pak};
use egui::text_edit;
use std::fs::{rename, File};
use std::io::Write;
use std::path::PathBuf;
use std::{env, fs};

#[derive(Debug)]
pub struct ModBuilder {
    building_multi_pak: bool,
    working_dir: PathBuf,
    temp_dir: PathBuf,
    img_dir: PathBuf,
    pak_dir: PathBuf,
    modification: ModType,
}

fn toggle(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), *on, "")
    });

    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool_responsive(response.id, *on);
        let visuals = ui.style().interact_selectable(&response, *on);
        let rect = rect.expand(visuals.expansion);
        let radius = 0.5 * rect.height();
        ui.painter()
            .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
        let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
        let center = egui::pos2(circle_x, rect.center().y);
        ui.painter()
            .circle(center, 0.75 * radius, visuals.bg_fill, visuals.fg_stroke);
    }

    response
}
impl ModBuilder {
    pub fn new() -> Self {
        Self {
            building_multi_pak: false,
            working_dir: env::current_dir().unwrap(),
            temp_dir: env::current_dir().unwrap().join("temp"),
            img_dir: env::current_dir().unwrap().join("temp").join("images"),
            pak_dir: env::current_dir().unwrap().join("temp").join("paks"),
            modification: ModType::Complete(Modification::new()),
        }
    }
    pub fn check_dirs_multipak(&mut self) {
        if !self.temp_dir.exists() {
            fs::create_dir_all(&self.temp_dir).unwrap();
        }
        if !self.img_dir.exists() {
            fs::create_dir_all(&self.img_dir).unwrap();
        }
        if !self.pak_dir.exists() {
            fs::create_dir_all(&self.pak_dir).unwrap();
        }
    }
    pub fn check_dirs_complete(&mut self) {
        if !self.temp_dir.exists() {
            fs::create_dir_all(&self.temp_dir).unwrap();
        }
        if self.img_dir.exists() {
            fs::remove_dir_all(&self.img_dir).unwrap();
        }
        if self.pak_dir.exists() {
            fs::remove_dir_all(&self.pak_dir).unwrap();
        }
    }
}
impl eframe::App for ModBuilder {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Build").clicked() {
                    match &self.modification {
                        ModType::Complete(_modification) => {
                            self.check_dirs_complete();
                        }
                        ModType::MultiPak(_modification) => {
                            self.check_dirs_multipak();
                        }
                        _ => {}
                    }
                    match &self.modification {
                        ModType::Complete(modification) => {
                            match serde_json::to_string_pretty(&modification) {
                                Ok(json) => {
                                    if !modification.name.is_empty() {
                                        let mut file = File::create("temp/mod.json").unwrap();
                                        file.write_all(json.as_bytes()).unwrap();
                                        drop(file);
                                        rename(
                                            &self.temp_dir.as_path(),
                                            &self.working_dir.join(&modification.name),
                                        )
                                        .unwrap();
                                    }
                                }
                                Err(e) => eprintln!("{}", e),
                            }
                        }
                        ModType::MultiPak(modification) => {
                            match serde_json::to_string_pretty(&modification) {
                                Ok(json) => {
                                    if !modification.name.is_empty() {
                                        let mut file = File::create("temp/mod.json").unwrap();
                                        file.write_all(json.as_bytes()).unwrap();
                                        drop(file);
                                        rename(
                                            &self.temp_dir.as_path(),
                                            &self.working_dir.join(&modification.name),
                                        )
                                        .unwrap();
                                    }
                                }
                                Err(e) => eprintln!("{}", e),
                            }
                        }
                        _ => {}
                    }
                }
            })
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
                match &self.modification {
                    ModType::Complete(_modification) => {
                        self.check_dirs_complete();
                    }
                    ModType::MultiPak(_modification) => {
                        self.check_dirs_multipak();
                    }
                    _ => {}
                }
                match &mut self.modification {
                    ModType::Complete(modification) => {
                        let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
                        println!("File Dropped: {:?}", dropped_files);
                        if !self.working_dir.join("temp").exists() {
                            fs::create_dir_all(&self.working_dir.join("temp")).unwrap();
                        }
                        for file in dropped_files {
                            let Some(filepath) = &file.path else {
                                panic!("Failed to find file in dropped_files")
                            };
                            let filename = filepath.file_name().unwrap().to_str().unwrap();
                            if filename.ends_with(".png") || filename.ends_with(".jpg") {
                                let destination = self.working_dir.join("temp").join(filename);
                                fs::rename(filepath, destination).unwrap();
                                modification.images.push(PathBuf::from(filename));
                            } else if filename.ends_with(".pak") {
                                modification.name = filename.to_string();
                                let destination = self.working_dir.join("temp").join(filename);
                                fs::rename(filepath, destination).unwrap();
                            }
                        }
                    }
                    ModType::MultiPak(modification) => {
                        let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
                        println!("File Dropped: {:?}", dropped_files);
                        if !self.working_dir.join("temp").exists() {
                            fs::create_dir_all(&self.working_dir.join("temp")).unwrap();
                        }
                        for file in dropped_files {
                            let Some(filepath) = &file.path else {
                                panic!("Failed to find file in dropped_files")
                            };
                            let filename = filepath.file_name().unwrap().to_str().unwrap();
                            if filename.ends_with(".png") || filename.ends_with(".jpg") {
                                let destination = self.img_dir.join(filename);
                                fs::rename(filepath, destination).unwrap();
                            } else if filename.ends_with(".pak") {
                                let destination = self.pak_dir.join(filename);
                                fs::rename(filepath, destination).unwrap();
                                let mut pak = Pak::new(filename.to_string());
                                pak.set_name(filename.to_string());
                                modification.paks.push(pak);
                            }
                        }
                    }
                    _ => {}
                }
                println!("{:?}", self);
            }

            ui.heading("Mod Builder");
            ui.add_space(10.0);

            // Custom sliding switch
            ui.horizontal(|ui| {
                ui.label("Regular Mod");
                if toggle(ui, &mut self.building_multi_pak).changed() {
                    if self.building_multi_pak {
                        println!("Switch to MultiPak");
                        self.modification = ModType::MultiPak(MultiPak::default());
                        self.check_dirs_complete();
                        fs::remove_dir_all(&self.working_dir.join("temp"))
                            .expect("failed to find temp dir");
                    } else {
                        println!("Switch to Single Pak");
                        self.modification = ModType::Complete(Modification::new());
                        self.check_dirs_complete();
                        fs::remove_dir_all(&self.working_dir.join("temp"))
                            .expect("failed to find temp dir");
                    }
                }
                ui.label("MultiPak Mod");
            });
            ui.separator();
            match &mut self.modification {
                ModType::Complete(ref mut cm) => {
                    ui.label(
                        "Drag and drop .pak and image files on this window to add them to the mod.",
                    );
                    ui.horizontal(|ui| {
                        ui.label("Name: ");
                        if ui
                            .add(text_edit::TextEdit::singleline(&mut cm.name))
                            .changed()
                        {
                            println!("{}", cm.name);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Description: ");
                        if ui
                            .add(text_edit::TextEdit::singleline(&mut cm.description))
                            .changed()
                        {
                            println!("{}", cm.description);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Author: ");
                        if ui
                            .add(text_edit::TextEdit::singleline(&mut cm.author))
                            .changed()
                        {
                            println!("{}", cm.author);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Version: ");
                        if ui
                            .add(text_edit::TextEdit::singleline(&mut cm.version))
                            .changed()
                        {
                            println!("{}", cm.version);
                        }
                    });
                }
                ModType::MultiPak(mp) => {
                    ui.label("Drag and drop .pak files on this window to add them to the mod.");
                    ui.horizontal(|ui| {
                        ui.label("Name: ");
                        if ui
                            .add(text_edit::TextEdit::singleline(&mut mp.name))
                            .changed()
                        {
                            println!("{}", mp.name);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Description: ");
                        if ui
                            .add(text_edit::TextEdit::singleline(&mut mp.description))
                            .changed()
                        {
                            println!("{}", mp.description);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Author: ");
                        if ui
                            .add(text_edit::TextEdit::singleline(&mut mp.author))
                            .changed()
                        {
                            println!("{}", mp.author);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Version: ");
                        if ui
                            .add(text_edit::TextEdit::singleline(&mut mp.version))
                            .changed()
                        {
                            println!("{}", mp.version);
                        }
                    });
                    ui.heading("Paks:");
                    ui.separator();

                    if self.pak_dir.exists() {
                        for pak in mp.paks.iter_mut() {
                            ui.heading(format!("Pak {}", pak.pak));
                            ui.horizontal(|ui| {
                                ui.label("Name: ");
                                if ui
                                    .add(text_edit::TextEdit::singleline(&mut pak.name))
                                    .changed()
                                {
                                    println!("{}", pak.name);
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("Description: ");
                                if ui
                                    .add(text_edit::TextEdit::singleline(&mut pak.description))
                                    .changed()
                                {
                                    println!("{}", pak.description);
                                }
                            });
                            ui.label("Choose images to be associated with .pak: ");
                            for image in self.img_dir.read_dir().unwrap() {
                                let image =
                                    PathBuf::from(image.unwrap().file_name().to_str().unwrap());
                                ui.horizontal(|ui| {
                                    ui.label(image.to_str().unwrap());
                                    let mut is_selected = pak.images.contains(&image);
                                    if ui
                                        .checkbox(&mut is_selected, &*image.to_str().unwrap())
                                        .clicked()
                                    {
                                        if is_selected {
                                            pak.images.push(PathBuf::from(image));
                                        } else {
                                            pak.images.retain(|p| p.to_str() != image.to_str());
                                        }
                                    }
                                });
                            }
                        }
                    }
                }
                _ => {
                    eprintln!("Unknown ModType");
                }
            }
        });
    }
}
