/// An asset that can be resolved into binary data
pub trait ResolvableAsset {
    fn resolve(&self) -> Vec<u8>;
    fn get_name(&self) -> &str;
}

/// An asset that is stored on disk
#[derive(Debug, Clone)]
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

/// An asset stored in memory
#[derive(Debug, Clone)]
pub struct MemoryAsset {
    data: Vec<u8>,
    name: String,
}

impl MemoryAsset {
    pub fn new(data: Vec<u8>, name: &str) -> Self {
        MemoryAsset {
            data,
            name: name.to_string(),
        }
    }
}

impl ResolvableAsset for MemoryAsset {
    fn resolve(&self) -> Vec<u8> {
        self.data.clone()
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}
