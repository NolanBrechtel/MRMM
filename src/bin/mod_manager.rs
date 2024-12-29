#![windows_subsystem = "windows"]

use egui::ViewportBuilder;
use marvel_rivals_mod_manager::tools::*;
use std::env;
use std::path::PathBuf;
fn main() -> eframe::Result {
    let mut manager: ModManager = ModManager::new();
    manager.mod_directory = env::current_dir().unwrap().join("mods");
    manager.game_directory = find_game_dir();
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

fn find_game_dir() -> PathBuf {
    let steam_dir = steamlocate::SteamDir::locate().unwrap();
    let (marvel_rivals, _lib) = steam_dir
        .find_app(2767030)
        .unwrap()
        .expect("Marvel Rivals not found");
    assert_eq!(marvel_rivals.name.as_ref().unwrap(), "Marvel Rivals");
    println!("{:?}", marvel_rivals);
    for library in steam_dir.libraries().unwrap() {
        let library = library.unwrap();
        println!("{:?}", library);
        for app in library.apps() {
            let app = app.unwrap();
            if app.app_id == marvel_rivals.app_id {
                return library
                    .path()
                    .join("steamapps")
                    .join("common")
                    .join(marvel_rivals.install_dir);
            }
        }
    }
    PathBuf::new()
}
