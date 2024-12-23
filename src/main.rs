mod mod_types;
mod tools;

use std::env;
use std::path::Path;
use egui::ViewportBuilder;
use tools::*;
use image::{io::Reader as ImageReader, ImageFormat};
use winit::window::Icon;


fn main() -> eframe::Result {
    let mut manager: ModManager = ModManager::new();
    manager.mod_directory = env::current_dir().unwrap().join("mods");
    manager.init_mods();
    println!("{:?}", manager);

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

    eframe::run_native(
        "Mod Manager",
        native_options,
        Box::new(|_cc| Ok(Box::new(manager))),
    )
}
fn load_icon(path: &str) -> Result<Icon, Box<dyn std::error::Error>> {
    // Load the image from the specified path
    let img = image::ImageReader::open(Path::new(path))?
        .with_guessed_format()?
        .decode()?
        .to_rgba8();

    let (width, height) = img.dimensions();
    let rgba = img.into_raw();
    Ok(Icon::from_rgba(rgba, width, height)?)
}