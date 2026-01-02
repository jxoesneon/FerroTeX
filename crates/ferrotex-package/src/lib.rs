use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod scanner;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackageIndex {
    pub packages: HashMap<String, PackageMetadata>,
}

impl PackageIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, name: String, metadata: PackageMetadata) {
        self.packages.insert(name, metadata);
    }

    pub fn get(&self, name: &str) -> Option<&PackageMetadata> {
        self.packages.get(name)
    }

    /// Returns the default cache file path: ~/.cache/ferrotex/packages.json
    pub fn cache_path() -> Option<std::path::PathBuf> {
        dirs::cache_dir().map(|p| p.join("ferrotex").join("packages.json"))
    }

    /// Saves the index to the cache file.
    pub fn save_to_cache(&self) -> std::io::Result<()> {
        if let Some(path) = Self::cache_path() {
            self.save_to_path(&path)?;
            log::info!("Saved package index to {:?}", path);
        }
        Ok(())
    }

    pub fn save_to_path(&self, path: &std::path::Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Loads the index from the cache file, if it exists.
    pub fn load_from_cache() -> Option<Self> {
        let path = Self::cache_path()?;
        Self::load_from_path(&path)
    }

    pub fn load_from_path(path: &std::path::Path) -> Option<Self> {
        if path.exists() {
            match std::fs::read_to_string(path) {
                Ok(content) => match serde_json::from_str::<PackageIndex>(&content) {
                    Ok(index) => {
                        let packages_len = index.packages.len();
                        log::info!("Cache hit. Loaded {} packages from cache.", packages_len);
                        return Some(index);
                    }
                    Err(e) => log::warn!("Failed to parse index: {}", e),
                },
                Err(e) => log::warn!("Failed to read index: {}", e),
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_index_persistence() {
        let mut index = PackageIndex::new();
        let mut meta = PackageMetadata::default();
        meta.commands.push("test".to_string());
        index.insert("mypkg".to_string(), meta);

        let temp_dir = std::env::current_dir()
            .unwrap()
            .join("target")
            .join("test_cache_2");
        let temp_file = temp_dir.join("packages.json");

        index.save_to_path(&temp_file).unwrap();
        assert!(temp_file.exists());

        let loaded = PackageIndex::load_from_path(&temp_file).unwrap();
        assert_eq!(loaded.packages.len(), 1);

        // Test load from non-existent path
        let non_existent = temp_dir.join("missing.json");
        assert!(PackageIndex::load_from_path(&non_existent).is_none());

        // Test load invalid JSON
        let invalid_file = temp_dir.join("invalid.json");
        std::fs::write(&invalid_file, "{ invalid }").unwrap();
        assert!(PackageIndex::load_from_path(&invalid_file).is_none());

        // Cleanup
        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_cache_path() {
        // Just verify it returns something or None without crashing
        let _ = PackageIndex::cache_path();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackageMetadata {
    pub commands: Vec<String>,
    pub environments: Vec<String>,
}
