use std::path::PathBuf;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize, Serialize)]
struct Pak {
    pub name: String,
    pub description: String,
    pub images: Vec<PathBuf>,
    pub pak: String,
    #[serde(skip)]
    pub path: PathBuf,
}
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct MultiPak {
    author: String,
    description: String,
    version: String,
    name: String,
    paks: Vec<Pak>,
    #[serde(skip)]
    enabled: bool,
    #[serde(skip)]
    path: PathBuf,
    #[serde(skip)]
    selected_pak: usize,
}

impl MultiPak {
    pub fn build(path: PathBuf) -> Result<Self, String> {
        let json_path: PathBuf = path.join("mod.json");
        let json_content = std::fs::read_to_string(&json_path).unwrap();
        let multi_pak: Result<MultiPak, serde_json::Error> = serde_json::from_str(&json_content);
        let mut modification = multi_pak.unwrap();
        for pak in &mut modification.paks {
            pak.path = path.join("paks").join(&pak.name)
        }
        modification.path = path;
        modification.enabled = false;
        Ok(modification)
    }
    pub fn selected_pak(&self) -> &Pak {
        &self.paks[self.selected_pak]
    }
}