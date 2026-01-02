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
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let json = serde_json::to_string_pretty(self)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            std::fs::write(&path, json)?;
            log::info!("Saved package index to {:?}", path);
        }
        Ok(())
    }

    /// Loads the index from the cache file, if it exists.
    pub fn load_from_cache() -> Option<Self> {
        let path = Self::cache_path()?;
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    match serde_json::from_str::<PackageIndex>(&content) {
                        Ok(index) => {
                            let packages_len = index.packages.len();
                            log::info!("Cache hit. Loaded {} packages from cache.", packages_len);
                            return Some(index);
                        }
                        Err(e) => log::warn!("Failed to parse cache: {}", e),
                    }
                }
                Err(e) => log::warn!("Failed to read cache: {}", e),
            }
        }
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackageMetadata {
    pub commands: Vec<String>,
    pub environments: Vec<String>,
}
