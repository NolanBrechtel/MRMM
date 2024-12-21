#[derive(Debug, Default)]
pub struct Pak {
    name: String,
    path: String,
}
impl Pak {
    fn new() -> Self {
        Self::default()
    }
}