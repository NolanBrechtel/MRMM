use std::path::PathBuf;
use crate::mod_types::*;
use std::fmt::Debug;
use std::fs;
use crate::mod_types::ModType::*;

#[derive(Default, Debug)]
pub struct ModManager {
    modifications: Vec<ModType>,
    pub mod_directory: PathBuf,
    game_directory: PathBuf,
}


impl ModManager {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn clear_mod_directory(&self) {
        fs::remove_dir_all(&self.mod_directory).unwrap();
    }
    pub fn load_mods(&mut self) {
        let game_mod_directory = self.game_directory.join(r"MarvelGame\Marvel\Content\Paks\~mods");
        fs::create_dir_all(&game_mod_directory).unwrap_or_else(|err| panic!("Failed to create directory {:?}: {}", game_mod_directory, err));
        for mod_type in &self.modifications {
            match mod_type {
                LoosePak(lp) => {
                    let destination_path = game_mod_directory.join(&lp.path().file_name().unwrap());
                    fs::copy(&lp.path(), &destination_path).unwrap_or_else(|err| panic!("Failed to copy {:?}: {}", &lp.path(), err));
                }
                Complete(cm) => {
                    let destination_path = game_mod_directory.join(&cm.pak_path().file_name().unwrap());
                    fs::copy(&cm.pak_path(), &destination_path).unwrap_or_else(|err| panic!("Failed to copy {:?}: {}", &cm.pak_path(), err));
                }
                MultiPak(mp) => {
                    let destination_path = game_mod_directory.join(&mp.selected_pak().pak);
                    fs::copy(&mp.selected_pak().path, &destination_path).unwrap_or_else(|err| panic!("Failed to copy {:?}: {}", &mp.selected_pak().path, err));
                }
            }
        }
    }
    pub fn init_mods(&mut self) {
        if let Ok(entries) = std::fs::read_dir(&self.mod_directory) {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        if path.is_dir(){
                            if path.join("paks").exists() {
                                println!("Building MultiPak from {}", path.to_str().unwrap());
                                match MultiPak::build(path) {
                                    Ok(multipak) => self.modifications.push(ModType::MultiPak(multipak)), // Push the MultiPak if build succeeds
                                    Err(err) => eprintln!("Failed to build MultiPak: {}", err), // Log the error if it fails
                                }
                            } else {
                                println!("Building Complete mod from {}", path.to_str().unwrap());
                                match Modification::build(path) {

                                    Ok(modification) => self.modifications.push(ModType::Complete(modification)), // Build a complete mod if successful
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