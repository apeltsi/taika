pub trait ResolvableAsset {
    fn resolve(&self) -> Vec<u8>;
    fn get_name(&self) -> &str;
}

pub struct DiskAsset {
    path: String,
}

impl DiskAsset {
    pub fn new(path: &str) -> Self {
        DiskAsset {
            path: path.to_string(),
        }
    }
}

impl ResolvableAsset for DiskAsset {
    fn resolve(&self) -> Vec<u8> {
        let path = std::env::current_exe().unwrap();
        let path = path.parent().unwrap();
        let path = path.join(&self.path);
        std::fs::read(path).unwrap()
    }

    fn get_name(&self) -> &str {
        &self.path
    }
}
