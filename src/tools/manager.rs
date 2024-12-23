use crate::mod_types::ModType::*;
use crate::mod_types::MultiPak;
use crate::mod_types::LoosePak;
use crate::mod_types::*;
use std::fmt::Debug;
use std::fs;
use std::path::PathBuf;
use eframe::Frame;
use egui::Context;

#[derive(Default, Debug)]
pub struct ModManager {
    modifications: Vec<ModType>,
    pub mod_directory: PathBuf,
    game_directory: PathBuf,
    mod_load_status: String,
    selected_mod_index: Option<usize>,
}

impl ModManager {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn validate_game_directory(&self) -> bool {
        if self.game_directory.exists() {
            if self.game_directory.join("MarvelGame").exists() {
                return true;
            }
        }
        return false;
    }
    pub fn clear_mod_directory(&self) {
        let game_mod_directory = self
            .game_directory
            .join(r"MarvelGame\Marvel\Content\Paks\~mods");
        fs::remove_dir_all(&game_mod_directory).unwrap_or_else(|err| {
            eprintln!(
                "Failed to remove directory {:?}: {}",
                game_mod_directory, err
            )
        });
    }
    pub fn load_mods(&mut self) {
        if !self.validate_game_directory() {
            eprintln!("Game directory is invalid.");
            return;
        }
        let game_mod_directory = self
            .game_directory
            .join(r"MarvelGame\Marvel\Content\Paks\~mods");
        fs::create_dir_all(&game_mod_directory).unwrap_or_else(|err| {
            eprintln!(
                "Failed to create directory {:?}: {}",
                game_mod_directory, err
            )
        });
        for mod_type in &self.modifications {
            match mod_type {
                LoosePak(lp) => {
                    if lp.enabled{
                        let destination_path = game_mod_directory.join(&lp.path().file_name().unwrap());
                        fs::copy(&lp.path(), &destination_path)
                            .unwrap_or_else(|err| panic!("Failed to copy {:?}: {}", &lp.path(), err));
                    }
                }
                Complete(cm) => {
                    if cm.enabled {
                        let destination_path = game_mod_directory.join(&cm.pak_path().file_name().unwrap());
                        fs::copy(&cm.pak_path(), &destination_path).unwrap_or_else(|err| {
                            panic!("Failed to copy {:?}: {}", &cm.pak_path(), err)
                        });
                    }
                }
                MultiPak(mp) => {
                    if mp.enabled {
                        let destination_path = game_mod_directory.join(&mp.selected_pak().pak);
                        fs::copy(&mp.selected_pak().path, &destination_path).unwrap_or_else(|err| {
                            panic!("Failed to copy {:?}: {}", &mp.selected_pak().path, err)
                        });
                    }
                }
            }
        }
    }
    pub fn init_mods(&mut self) {
        self.modifications.clear();
        if let Ok(entries) = std::fs::read_dir(&self.mod_directory) {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        if path.is_dir() {
                            if path.join("paks").exists() {
                                println!("Building MultiPak from {}", path.to_str().unwrap());
                                match MultiPak::build(path) {
                                    Ok(multipak) => {
                                        self.modifications.push(ModType::MultiPak(multipak))
                                    } // Push the MultiPak if build succeeds
                                    Err(err) => eprintln!("Failed to build MultiPak: {}", err), // Log the error if it fails
                                }
                            } else {
                                println!("Building Complete mod from {}", path.to_str().unwrap());
                                match Modification::build(path) {
                                    Ok(modification) => {
                                        self.modifications.push(ModType::Complete(modification))
                                    } // Build a complete mod if successful
                                    Err(err) => eprintln!("Failed to build modification: {}", err), // Log the error if it fails
                                }
                            }
                        } else if path.to_str().unwrap().ends_with(".pak") {
                            println!("Building LoosePak from {}", path.to_str().unwrap());
                            match LoosePak::build(path) {
                                Ok(pak) => self.modifications.push(ModType::LoosePak(pak)),
                                Err(err) => eprintln!("Failed to build Pak: {}", err),
                            }
                        } else {
                            println!("{:?} is not a mod file type.", path)
                        }
                    }
                    Err(err) => {
                        eprintln!("Failed to read mod directory: {}", err);
                    }
                }
            }
        } else {
            eprintln!("Failed to read mod directory {:?}", self.mod_directory)
        }
    }
}

impl eframe::App for ModManager {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Mod Manager");
            ui.horizontal(|ui| {
                ui.label("Game Directory: ");
                let mut game_directory_str = self.game_directory.to_str().unwrap_or("").to_string();
                let text_edit = egui::TextEdit::singleline(&mut game_directory_str).clip_text(false);
                if ui.add(text_edit).changed() {
                    self.game_directory = std::path::PathBuf::from(game_directory_str);
                }
                if self.validate_game_directory() {
                    ui.label(egui::RichText::new("Valid Game Directory").color(egui::Color32::GREEN))
                } else {
                    ui.label(egui::RichText::new("Invalid Game Directory").color(egui::Color32::RED))
                };
            });
            ui.separator();
            ui.label(format!("Stored Mods Directory:   {}", self.mod_directory.to_str().unwrap_or("")));
            ui.horizontal(|ui| {
                ui.label("Game's Mod Directory: ");
                ui.label(self.game_directory.join(r"MarvelGame\Marvel\Content\Paks\~mods").to_str().unwrap_or("").to_string());
            });
            ui.horizontal(|ui| {
                if ui.button("Refresh Mods").clicked() {
                    self.init_mods();
                }
                if ui.button("Load Mods").clicked() {
                    if self.validate_game_directory() {
                        self.load_mods();
                        self.mod_load_status = "success".to_string();
                    } else {
                        self.mod_load_status = "failed".to_string();
                    }
                }
                if self.mod_load_status == "success" {
                    ui.label(egui::RichText::new("Successfully loaded mods. Launch game through Steam").color(egui::Color32::GREEN));
                } else if self.mod_load_status == "failed" {
                    ui.label(egui::RichText::new("Failed to load mods. Check the Game Directory.").color(egui::Color32::RED));
                }
            });
            ui.separator();
            ui.columns(2, |columns| {
                columns[0].heading("Available Mods");
                for (index, modification) in self.modifications.iter_mut().enumerate() {
                    columns[0].horizontal(|ui| {
                        match modification {
                            LoosePak(lp) =>{
                                let mut enabled = lp.enabled;
                                if ui.checkbox(&mut enabled, "").changed() {
                                    lp.enabled = enabled;
                                }
                                if ui.selectable_label(
                                    self.selected_mod_index == Some(index),
                                    format!("{}", lp.name)
                                ).clicked() {
                                    self.selected_mod_index = Some(index);
                                }
                            }
                            Complete(cm) =>{
                                let mut enabled = cm.enabled;
                                if ui.checkbox(&mut enabled, "").changed() {
                                    cm.enabled = enabled;
                                }
                                if ui.selectable_label(
                                    self.selected_mod_index == Some(index),
                                    format!("{} v{}", cm.name, cm.version)
                                ).clicked() {
                                    self.selected_mod_index = Some(index);
                                }
                            }
                            MultiPak(mp) =>{
                                let mut enabled = mp.enabled;
                                if ui.checkbox(&mut enabled, "").changed() {
                                    mp.enabled = enabled;
                                }
                                if ui.selectable_label(
                                    self.selected_mod_index == Some(index),
                                    format!("{} v{}", mp.name, mp.version)
                                ).clicked() {
                                    self.selected_mod_index = Some(index);
                                }
                            }
                        }
                    });
                }
                columns[1].heading("Mod Details:");
                if let Some(selected_index) = self.selected_mod_index {
                    let selected_mod = self.modifications.get_mut(selected_index).unwrap();
                    match selected_mod {
                        LoosePak(lp) => {
                            columns[1].label(format!("Name: {}", lp.name));
                        }
                        Complete(cm) => {
                            columns[1].label(format!("Name: {}", cm.name));
                            columns[1].label(format!("Version: {}", cm.version));
                            columns[1].label(format!("Author: {}", cm.author));
                            columns[1].label(format!("Description: {}", cm.description));
                        }
                        MultiPak(mp) => {
                            columns[1].label(format!("Name: {}", mp.name));
                            columns[1].label(format!("Version: {}", mp.version));
                            columns[1].label(format!("Author: {}", mp.author));
                            columns[1].label(format!("Description: {}", mp.description));
                            columns[1].separator();
                            columns[1].horizontal(|mut ui| {
                                ui.heading("Select which pak to load:");
                                egui::ComboBox::from_label("")
                                    .selected_text(format!("{}", mp.selected_pak().name))
                                    .show_ui(&mut ui, |ui| {
                                        for (index, pak ) in mp.paks.iter().enumerate() {
                                            if ui.selectable_label(
                                                mp.selected_pak().name == pak.name,
                                                format!("{}", pak.name)
                                            ).clicked() {
                                                mp.selected_pak = index;
                                                println!("{:?}", mp.selected_pak());
                                            }
                                        }
                                    });
                            });
                            columns[1].label(format!("Name: {}", mp.selected_pak().name));
                            columns[1].label(format!("Description: {}", mp.selected_pak().description));
                        }
                    }
                } else {
                    columns[1].label("Select a mod to view its details.");
                }
            });
        });
    }
}