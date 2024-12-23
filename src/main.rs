mod mod_types;
mod tools;

use std::env;
use tools::*;

fn main() -> eframe::Result {
    let mut manager: ModManager = ModManager::new();
    manager.mod_directory = env::current_dir().unwrap().join("mods");
    manager.init_mods();
    println!("{:?}", manager);
    eframe::run_native(
        "Mod Manager",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(manager)))
    )
}
