use std::{fs, path::PathBuf};

pub struct FileBackedStorage {
    path: PathBuf,
}

fn ensure_parent_dir(path: &std::path::Path) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create_dir_all failed");
    }
}

impl FileBackedStorage {
    pub fn new(path: PathBuf) -> Self {
        ensure_parent_dir(&path);
        Self { path }
    }

    fn write_bytes(&self, bytes: &[u8]) {
        // à remplacer par "chiffrer(bytes)" puis write
        fs::write(&self.path, bytes).expect("write failed");
    }

    fn read_bytes(&self) -> Option<Vec<u8>> {
        fs::read(&self.path).ok()
    }
}
