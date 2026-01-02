use std::path::PathBuf;
use std::fs;
use sha2::{Sha256, Digest};
use crate::{Artifact, ArtifactId};

#[derive(Debug, Clone)]
pub struct FileArtifact {
    pub path: PathBuf,
}

impl FileArtifact {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Artifact for FileArtifact {
    fn id(&self) -> ArtifactId {
        // ID is path-based for source files, or content-based for intermediate?
        // For simplicity in FileArtifact, we use the absolute path string.
        let abs_path = fs::canonicalize(&self.path).unwrap_or(self.path.clone());
        ArtifactId(abs_path.to_string_lossy().to_string())
    }

    fn fingerprint(&self) -> String {
        match fs::read(&self.path) {
            Ok(bytes) => {
                let mut hasher = Sha256::new();
                hasher.update(&bytes);
                hex::encode(hasher.finalize())
            }
            Err(_) => "MISSING".to_string(), // Or handle error gracefully
        }
    }

    fn path(&self) -> Option<PathBuf> {
        Some(self.path.clone())
    }
}
