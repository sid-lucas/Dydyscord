use std::os::unix::fs::PermissionsExt;
use std::{env, fs, path::PathBuf};

pub struct FileBackedStorage {
    path: PathBuf,
}

fn ensure_storage_path() -> PathBuf {
    // chemin jusqu'au dossier
    let home = env::var("HOME").expect("HOME not set");

    let mut dir = PathBuf::from(home);
    dir.push(".dydyscord");

    // Créer le dossier de l'app si non existant
    if !dir.exists() {
        fs::create_dir_all(&dir).expect("Failed to create dir");
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
    }

    // chemin jusqu'au fichier secret
    let mut file = dir.clone();
    file.push(".secret");

    // Créer le fichier si non existant
    if !file.exists() {
        fs::File::create(&file).expect("Failed to create file");
        fs::set_permissions(&file, fs::Permissions::from_mode(0o600)).unwrap();
    }

    file
}

impl FileBackedStorage {
    pub fn new() -> Self {
        let path = ensure_storage_path();
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

pub fn test() -> Result<(), ()> {
    let s = FileBackedStorage::new();
    s.write_bytes(b"hello");
    println!("{:?}", s.read_bytes());

    Ok(())
}
