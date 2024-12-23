use serde_json;
use std::path::PathBuf;

#[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct Modification {
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub images: Vec<PathBuf>,
    #[serde(skip)]
    pub file_path: PathBuf,
    #[serde(skip)]
    pub enabled: bool,
}
impl Modification {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn pak_path(&self) -> PathBuf {
        let entries = self.file_path.read_dir().unwrap();
        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.to_str().unwrap().ends_with(".pak") {
                return path;
            }
        }
        panic!("No pak file found in {:?}", self.file_path);
    }
    pub fn build(file_path: PathBuf) -> Result<Self, String> {
        let json_path: PathBuf = file_path.join("mod.json");

        // Attempt to read the file; handle errors
        let json_content = match std::fs::read_to_string(&json_path) {
            Ok(content) => content,
            Err(err) => {
                return Err(format!(
                    "Failed to read JSON file at {:?}: {}",
                    json_path, err
                ))
            }
        };

        // Attempt to parse the JSON; handle errors
        let mut modification = match Self::from_json(json_content) {
            Ok(modification) => modification,
            Err(err) => {
                return Err(format!(
                    "Failed to parse JSON from file at {:?}: {}",
                    json_path, err
                ))
            }
        };

        modification.file_path = file_path;
        modification.enabled = false;

        Ok(modification)
    }
    pub fn from_json(json: String) -> Result<Self, serde_json::Error> {
        let modification: Result<Modification, serde_json::Error> = serde_json::from_str(&json);
        modification
    }
}
