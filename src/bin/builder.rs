#![windows_subsystem = "windows"]

use std::env;
use egui::ViewportBuilder;
use marvel_rivals_mod_manager::tools::*;


fn main() -> eframe::Result {
    let path_to_icon = env::current_dir().unwrap().join("icon.png");

    let loaded_icon = image::open(path_to_icon).unwrap();


    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder {
            // Set the icon field
            icon: Some(std::sync::Arc::new(egui::IconData {
                rgba: loaded_icon.to_rgba8().into_raw(),
                width: 64,
                height: 64,
            })),
            ..Default::default()
        },
        ..Default::default()
    };
    let mod_builder: ModBuilder = ModBuilder::new();
    eframe::run_native(
        "Mod Manager",
        native_options,
        Box::new(|_cc| Ok(Box::new(mod_builder))),
    )
}