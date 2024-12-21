use std::path::{Path, PathBuf};
use egui::Image;
use image::{DynamicImage, GenericImageView};
use serde_json;
use serde::Deserialize;

#[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct Modification {
    name: String,
    description: String,
    author: String,
    version: String,
    #[serde(skip)]
    file_path: PathBuf,
    #[serde(skip)]
    images: Vec<DynamicImage>,
    #[serde(skip)]
    enabled: bool,
}
impl Modification {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn build(&mut self, file_path: PathBuf) {
        let json_path: PathBuf = file_path.join("mod.json");
        let mut modification = Self::from_json(std::fs::read_to_string(json_path).unwrap());
        modification.populate_images(Path::new(&modification.file_path).join("images")); // Populates the images vector with all images in the images subfolder
        modification.file_path = file_path;
        modification.enabled = false;
    }
    pub fn add_image(&mut self, path: PathBuf) {

        if !path.exists() {
            return
        }
        match image::open(&path) {
            Ok(img) => {
                self.images.push(img);
            }
            Err(err) => {
                eprintln!("Failed to load image to DynamicImage {:?}: {}", path, err);
                return
            }
        }
    }
    pub fn populate_images(&mut self, path: PathBuf) {
        if !path.exists() {
            return
        }
        for entry in std::fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                self.add_image(path);
            }
        }
    }
    pub fn from_json(json: String) -> Self {
        let mut modification: Modification = serde_json::from_str(&json).unwrap(); // Gets author, description, version, and name from json file
        return modification
    }
}