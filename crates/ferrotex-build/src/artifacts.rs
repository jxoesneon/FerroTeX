use crate::{Artifact, ArtifactId};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

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
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_file_artifact_fingerprint() {
        let temp_dir = std::env::current_dir()
            .unwrap()
            .join("target")
            .join("test_artifacts");
        fs::create_dir_all(&temp_dir).unwrap();
        let file_path = temp_dir.join("test.txt");
        fs::write(&file_path, "hello world").unwrap();

        let artifact = FileArtifact::new(file_path.clone());
        let fp1 = artifact.fingerprint();

        assert_ne!(fp1, "MISSING");

        fs::write(&file_path, "modified").unwrap();
        let fp2 = artifact.fingerprint();
        assert_ne!(fp1, fp2);

        // Test missing file
        let _ = fs::remove_file(&file_path);
        assert_eq!(artifact.fingerprint(), "MISSING");

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_file_artifact_id_and_path() {
        let path = PathBuf::from("test.tex");
        let artifact = FileArtifact::new(path.clone());
        assert!(artifact.id().0.contains("test.tex"));
        assert_eq!(artifact.path().unwrap(), path);
    }
}
