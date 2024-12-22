use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct LoosePak {
    name: String,
    path: PathBuf,
    enabled: bool,
}
impl LoosePak {
    fn new() -> Self {
        Self::default()
    }
    pub fn build(path: PathBuf) -> Result<Self, String> {
        let mut pak = Self::new();
        pak.name = path.file_name().unwrap().to_str().unwrap().to_string();
        pak.path = path;
        pak.enabled = false;
        return Ok(pak)
    }
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}