use crate::mod_types::LoosePak;
use crate::mod_types::ModType::*;
use crate::mod_types::MultiPak;
use crate::mod_types::*;
use eframe::epaint::TextureHandle;
use eframe::Frame;
use egui::Context;
use image::GenericImageView;
use sevenz_rust::decompress_file as decompress_7z;
use std::fmt::Debug;
use std::fs;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::process::Command;
use zip::read::ZipArchive;

#[derive(Default, Debug)]
pub struct ModManager {
    modifications: Vec<ModType>,
    pub mod_directory: PathBuf,
    pub game_directory: PathBuf,
    mod_load_status: String,
    selected_mod_index: Option<usize>,
    current_image: usize,
}

impl ModManager {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn launch(&mut self) {
        let steam_dir = steamlocate::SteamDir::locate().unwrap();
        let steam_path = steam_dir.path().join("steam.exe");
        Command::new(steam_path)
            .arg("steam://rungameid/2767030")
            .spawn()
            .unwrap();
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
        self.clear_mod_directory();
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
                    if lp.enabled {
                        let destination_path =
                            game_mod_directory.join(&lp.path().file_name().unwrap());
                        fs::copy(&lp.path(), &destination_path).unwrap_or_else(|err| {
                            panic!("Failed to copy {:?}: {}", &lp.path(), err)
                        });
                    }
                }
                Complete(cm) => {
                    if cm.enabled {
                        let destination_path =
                            game_mod_directory.join(&cm.pak_path().file_name().unwrap());
                        fs::copy(&cm.pak_path(), &destination_path).unwrap_or_else(|err| {
                            panic!("Failed to copy {:?}: {}", &cm.pak_path(), err)
                        });
                    }
                }
                MultiPak(mp) => {
                    if mp.enabled {
                        let destination_path = game_mod_directory.join(&mp.selected_pak().pak);
                        fs::copy(&mp.selected_pak().path, &destination_path).unwrap_or_else(
                            |err| panic!("Failed to copy {:?}: {}", &mp.selected_pak().path, err),
                        );
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
                        if path.to_str().unwrap().ends_with(".7z") {
                            println!("Extracting 7z file {}", path.to_str().unwrap());
                            if let Err(err) = Self::extract_archive(&path) {
                                eprintln!("Failed to extract archive: {}", err);
                            }
                        } else if path.to_str().unwrap().ends_with(".zip") {
                            println!("Extracting zip file {}", path.to_str().unwrap());
                            if let Err(err) = Self::extract_archive(&path) {
                                eprintln!("Failed to extract archive: {}", err);
                            }
                        }
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
    fn add_mod(&mut self, file_path: &PathBuf) -> Result<(), std::io::Error> {
        // Ensure the target directory exists
        fs::create_dir_all(&self.mod_directory)?;

        // Build the destination path
        if let Some(file_name) = file_path.file_name() {
            let destination_path = self.mod_directory.join(file_name);

            // Move the file
            fs::rename(file_path, destination_path)?;
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "File has no valid name.",
            ));
        }

        Ok(())
    }
    fn load_image_to_texture(ctx: &egui::Context, path: &std::path::Path) -> Option<TextureHandle> {
        if !path.exists() {
            eprintln!("Image path does not exist: {:?}", path);
            return None;
        }

        // Load image dynamically at runtime
        match image::open(&path) {
            Ok(img) => {
                let (width, height) = img.dimensions();
                let rgba_image = img.to_rgba8();

                // Convert the loaded image into an egui texture
                let pixels = egui::ColorImage::from_rgba_unmultiplied(
                    [width as usize, height as usize],
                    rgba_image.as_flat_samples().as_slice(),
                );

                Some(ctx.load_texture(path.to_string_lossy(), pixels, Default::default()))
            }
            Err(err) => {
                eprintln!("Failed to load image {:?}: {}", path, err);
                None
            }
        }
    }
    pub fn extract_archive(file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        // Ensure the provided file path exists and is a file
        if !file_path.exists() || !file_path.is_file() {
            return Err("Provided file_path does not exist or is not a valid file.".into());
        }

        // Get the parent directory and stem name for output
        let parent_dir = file_path
            .parent()
            .ok_or("Unable to determine parent directory.")?;
        let output_dir = parent_dir;

        // Create the output directory if it doesn't exist
        if !output_dir.exists() {
            create_dir_all(&output_dir)?;
        }

        // Match based on file extension
        match file_path.extension().and_then(|ext| ext.to_str()) {
            Some("zip") => {
                let file = std::fs::File::open(file_path)?;
                let mut archive = ZipArchive::new(file)?;

                for i in 0..archive.len() {
                    let mut file = archive.by_index(i)?;
                    let out_path = output_dir.join(file.name());

                    if (&*file.name()).ends_with('/') {
                        // Directory entry
                        create_dir_all(&out_path)?;
                    } else {
                        // File entry
                        if let Some(parent) = out_path.parent() {
                            create_dir_all(parent)?;
                        }
                        let mut out_file = std::fs::File::create(out_path)?;
                        std::io::copy(&mut file, &mut out_file)?;
                    }
                }
            }
            Some("7z") => {
                decompress_7z(file_path, &output_dir)?;
            }
            _ => {
                return Err("Unsupported file format. Only .zip and .7z are supported.".into());
            }
        }

        println!(
            "Successfully extracted {:?} to {:?}",
            file_path.file_name().unwrap(),
            output_dir
        );

        Ok(())
    }
}
impl eframe::App for ModManager {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            // Set a horizontal layout and align items to the right
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Github").clicked() {
                        // Button clicked logic
                        open::that("https://github.com/NolanBrechtel/MRMM")
                            .expect("Failed to open Github");
                    }
                    if ui.button("Launch Game").clicked() {
                        self.launch();
                    }
                });
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
                // Get the files that were dropped into the window
                let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
                for dropped_file in dropped_files {
                    if let Some(path) = &dropped_file.path {
                        // Move the file to the target directory
                        if let Err(err) = self.add_mod(path) {
                            // Handle errors if the move fails
                            eprintln!("Failed to move file: {}", err);
                        } else {
                            println!("File successfully moved: {}", path.display());
                        }
                    } else {
                        // Handle files without paths (e.g., dragged from outside filesystem)
                        eprintln!("Dropped file has no valid path associated.");
                    }
                }
                self.init_mods();
            }
            ui.heading("Mod Manager");
            ui.horizontal(|ui| {
                ui.label("Game Directory: ");
                let mut game_directory_str = self.game_directory.to_str().unwrap_or("").to_string();
                let text_edit =
                    egui::TextEdit::singleline(&mut game_directory_str).clip_text(false);
                if ui.add(text_edit).changed() {
                    self.game_directory = std::path::PathBuf::from(game_directory_str);
                }
                if self.validate_game_directory() {
                    ui.label(
                        egui::RichText::new("Valid Game Directory").color(egui::Color32::GREEN),
                    )
                } else {
                    ui.label(
                        egui::RichText::new("Invalid Game Directory").color(egui::Color32::RED),
                    )
                };
            });
            ui.separator();
            ui.label(format!(
                "Stored Mods Directory:   {}",
                self.mod_directory.to_str().unwrap_or("")
            ));
            ui.horizontal(|ui| {
                ui.label("Game's Mod Directory: ");
                ui.label(
                    self.game_directory
                        .join(r"MarvelGame\Marvel\Content\Paks\~mods")
                        .to_str()
                        .unwrap_or("")
                        .to_string(),
                );
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
                    ui.label(
                        egui::RichText::new("Successfully loaded mods. Launch game through Steam")
                            .color(egui::Color32::GREEN),
                    );
                } else if self.mod_load_status == "failed" {
                    ui.label(
                        egui::RichText::new("Failed to load mods. Check the Game Directory.")
                            .color(egui::Color32::RED),
                    );
                }
            });
            ui.separator();
            ui.columns(2, |columns| {
                egui::ScrollArea::vertical().max_height(columns[0].available_height()).show(&mut columns[0], |ui| {
                    ui.horizontal(|ui| {
                        ui.heading("Available Mods");
                        if ui.button("Enable All").clicked() {
                            for modification in self.modifications.iter_mut() {
                                match modification {
                                    LoosePak(ref mut lp) => {
                                        lp.enabled = true;
                                    }
                                    Complete(ref mut cm) => {
                                        cm.enabled = true;
                                    }
                                    MultiPak(ref mut mp) => {
                                        mp.enabled = true;
                                    }
                                }
                            }
                        }
                        if ui.button("Disable All").clicked() {
                            for modification in self.modifications.iter_mut() {
                                match modification {
                                    LoosePak(ref mut lp) => {
                                        lp.enabled = false;
                                    }
                                    Complete(ref mut cm) => {
                                        cm.enabled = false;
                                    }
                                    MultiPak(ref mut mp) => {
                                        mp.enabled = false;
                                    }
                                }
                            }
                        }
                    });
                    for (index, modification) in self.modifications.iter_mut().enumerate() {
                        ui.horizontal(|ui| match modification {
                            LoosePak(lp) => {
                                let mut enabled = lp.enabled;
                                if ui.checkbox(&mut enabled, "").changed() {
                                    lp.enabled = enabled;
                                }
                                if ui
                                    .selectable_label(
                                        self.selected_mod_index == Some(index),
                                        format!("{}", lp.name),
                                    )
                                    .clicked()
                                {
                                    self.selected_mod_index = Some(index);
                                    self.current_image = 0;
                                }
                            }
                            Complete(cm) => {
                                let mut enabled = cm.enabled;
                                if ui.checkbox(&mut enabled, "").changed() {
                                    cm.enabled = enabled;
                                }
                                if ui
                                    .selectable_label(
                                        self.selected_mod_index == Some(index),
                                        format!("{}", cm.name),
                                    )
                                    .clicked()
                                {
                                    self.selected_mod_index = Some(index);
                                }
                            }
                            MultiPak(mp) => {
                                let mut enabled = mp.enabled;
                                if ui.checkbox(&mut enabled, "").changed() {
                                    mp.enabled = enabled;
                                }
                                if ui
                                    .selectable_label(
                                        self.selected_mod_index == Some(index),
                                        format!("{}", mp.name),
                                    )
                                    .clicked()
                                {
                                    self.selected_mod_index = Some(index);
                                }
                            }
                        });
                    }
                });

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
                            if !cm.images.is_empty() {
                                columns[1].separator();
                                columns[1].heading("Images:");
                                columns[1].horizontal(|ui| {
                                    // Left arrow to navigate to the previous image
                                    if ui.button("⬅").clicked() {
                                        if self.current_image == 0 {
                                            self.current_image = cm.images.len() - 1;
                                        } else {
                                            self.current_image -= 1;
                                        }
                                    }
                                    // Right arrow to navigate to the next image
                                    if ui.button("➡").clicked() {
                                        self.current_image =
                                            (self.current_image + 1) % cm.images.len();
                                    }
                                });
                                // Display the current image
                                if let Some(image_path) = cm.images.get(self.current_image) {
                                    // Dynamically load the image as a texture
                                    if let Some(texture) = Self::load_image_to_texture(
                                        ctx,
                                        &cm.file_path.join(image_path),
                                    ) {
                                        columns[1].add(
                                            egui::Image::new(&texture)
                                                .max_width(columns[1].available_width())
                                                .max_height(columns[1].available_height()),
                                        );
                                    } else {
                                        columns[1].label("Failed to load image.");
                                    }
                                }
                            }
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
                                        for (index, pak) in mp.paks.iter().enumerate() {
                                            if ui
                                                .selectable_label(
                                                    mp.selected_pak().name == pak.name,
                                                    format!("{}", pak.name),
                                                )
                                                .clicked()
                                            {
                                                mp.selected_pak = index;
                                                println!("{:?}", mp.selected_pak());
                                            }
                                        }
                                    });
                            });
                            columns[1].label(format!("Name: {}", mp.selected_pak().name));
                            columns[1]
                                .label(format!("Description: {}", mp.selected_pak().description));
                            if !mp.selected_pak().images.is_empty() {
                                columns[1].separator();
                                columns[1].heading("Images:");
                                columns[1].horizontal(|ui| {
                                    // Left arrow to navigate to the previous image
                                    if ui.button("⬅").clicked() {
                                        if self.current_image == 0 {
                                            self.current_image = mp.selected_pak().images.len() - 1;
                                        } else {
                                            self.current_image -= 1;
                                        }
                                    }
                                    // Right arrow to navigate to the next image
                                    if ui.button("➡").clicked() {
                                        self.current_image = (self.current_image + 1)
                                            % mp.selected_pak().images.len();
                                    }
                                });
                                // Display the current image
                                if let Some(image_path) =
                                    mp.selected_pak().images.get(self.current_image)
                                {
                                    // Dynamically load the image as a texture
                                    if let Some(texture) = Self::load_image_to_texture(
                                        ctx,
                                        &mp.path.join("images").join(image_path),
                                    ) {
                                        columns[1].add(
                                            egui::Image::new(&texture)
                                                .max_width(columns[1].available_width())
                                                .max_height(columns[1].available_height()),
                                        );
                                    } else {
                                        columns[1].label("Failed to load image.");
                                    }
                                }
                            }
                        }
                    }
                } else {
                    columns[1].label("Select a mod to view its details.");
                }
            });
        });
    }
}
