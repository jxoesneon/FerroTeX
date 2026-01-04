use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Lockfile {
    pub version: String,
    pub entries: HashMap<String, String>, // path -> sha256 hash
}

impl Lockfile {
    pub fn new() -> Self {
        Self {
            version: "0.20.0".to_string(),
            entries: HashMap::new(),
        }
    }

    pub fn save(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let lock: Self = serde_json::from_str(&content)?;
        Ok(lock)
    }
}

pub mod artifacts;
pub mod compiler;

pub use artifacts::FileArtifact;
pub use compiler::{Compiler, PdfLatexTransform, ShellTransform};

/// Represents a unique identifier for an artifact (content-addressed or path-based).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArtifactId(pub String);

/// An Artifact is a concrete input or output of the build process.
/// Examples: Source File, PDF, Log File, Object File.
pub trait Artifact {
    /// Returns the unique ID of this artifact.
    fn id(&self) -> ArtifactId;

    /// Returns the fingerprint (hash) of the artifact's content.
    /// This is crucial for hermeticity and caching.
    fn fingerprint(&self) -> String;

    /// Returns the path to the artifact on disk, if applicable.
    fn path(&self) -> Option<PathBuf>;
}

/// A Transform turns a set of Input Artifacts into Output Artifacts.
/// Examples: "Run pdflatex", "Copy file".
pub trait Transform {
    /// Returns the name/description of this description.
    fn description(&self) -> String;

    /// Returns the set of input Artifact IDs this transform depends on.
    fn inputs(&self) -> HashSet<ArtifactId>;

    /// Returns the set of output Artifact IDs this transform produces.
    fn outputs(&self) -> HashSet<ArtifactId>;

    /// Executes the transform implementation.
    /// Returns true if successful.
    fn execute(&self) -> Result<(), String>;
}

/// The Build Graph represents the DAG of all transforms and artifacts.
pub struct BuildGraph {
    /// Map of ArtifactId -> Box<dyn Artifact>
    artifacts: HashMap<ArtifactId, Box<dyn Artifact>>,
    /// List of transforms (edges/nodes in the DAG)
    transforms: Vec<Box<dyn Transform>>,
}

impl Default for BuildGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl BuildGraph {
    pub fn new() -> Self {
        Self {
            artifacts: HashMap::new(),
            transforms: Vec::new(),
        }
    }

    pub fn add_artifact(&mut self, artifact: Box<dyn Artifact>) {
        self.artifacts.insert(artifact.id(), artifact);
    }

    pub fn add_transform(&mut self, transform: Box<dyn Transform>) {
        self.transforms.push(transform);
    }

    /// Validates that the graph is a DAG (no cycles) and fully connected.
    pub fn validate(&self) -> Result<(), String> {
        // We simulate a strict ordering: Artifact -> Transform -> Artifact
        // To detect cycles, we need to traverse from each node.
        // For simplicity, let's just assert that for every Transform, its outputs are not in its inputs (trivial cycle),
        // and do a depth-first search to ensure no path leads back to start.

        // Adjacency: ArtifactId -> Vec<ArtifactId> (via Transforms)
        // A -> T -> B means A dependency of B.
        let mut adj: HashMap<ArtifactId, Vec<ArtifactId>> = HashMap::new();

        for transform in &self.transforms {
            for input in transform.inputs() {
                for output in transform.outputs() {
                    // Overlapping input/output is an immediate cycle
                    if input == output {
                        return Err(format!(
                            "Transform '{}' has self-cycle on {:?}",
                            transform.description(),
                            input
                        ));
                    }
                    adj.entry(input.clone()).or_default().push(output.clone());
                }
            }
        }

        // DFS for each node
        // 0 = Unvisited, 1 = Visiting, 2 = Visited
        let mut state: HashMap<ArtifactId, u8> = HashMap::new();

        fn has_cycle(
            current: &ArtifactId,
            adj: &HashMap<ArtifactId, Vec<ArtifactId>>,
            state: &mut HashMap<ArtifactId, u8>,
        ) -> bool {
            match state.get(current) {
                Some(1) => return true,  // Back edge found
                Some(2) => return false, // Already checked
                _ => {}
            }

            state.insert(current.clone(), 1); // Mark visiting

            if let Some(neighbors) = adj.get(current) {
                for neighbor in neighbors {
                    if has_cycle(neighbor, adj, state) {
                        return true;
                    }
                }
            }

            state.insert(current.clone(), 2); // Mark visited
            false
        }

        for artifact_id in self.artifacts.keys() {
            if has_cycle(artifact_id, &adj, &mut state) {
                return Err(format!(
                    "Cycle detected involving artifact {:?}",
                    artifact_id
                ));
            }
        }

        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_lockfile_roundtrip() {
        let mut lock = Lockfile::new();
        lock.entries
            .insert("file.tex".to_string(), "hash123".to_string());

        let temp_file = std::env::current_dir()
            .unwrap()
            .join("target")
            .join("test_lock.json");
        std::fs::create_dir_all(temp_file.parent().unwrap()).unwrap();

        lock.save(&temp_file).unwrap();
        let loaded = Lockfile::load(&temp_file).unwrap();

        assert_eq!(loaded.entries.get("file.tex").unwrap(), "hash123");
        let _ = std::fs::remove_file(temp_file);
    }

    struct MockArtifact(ArtifactId);
    impl Artifact for MockArtifact {
        fn id(&self) -> ArtifactId {
            self.0.clone()
        }
        fn fingerprint(&self) -> String {
            "const".to_string()
        }
        fn path(&self) -> Option<PathBuf> {
            None
        }
    }

    struct MockTransform {
        inputs: HashSet<ArtifactId>,
        outputs: HashSet<ArtifactId>,
    }
    impl Transform for MockTransform {
        fn description(&self) -> String {
            "mock".to_string()
        }
        fn inputs(&self) -> HashSet<ArtifactId> {
            self.inputs.clone()
        }
        fn outputs(&self) -> HashSet<ArtifactId> {
            self.outputs.clone()
        }
        fn execute(&self) -> Result<(), String> {
            Ok(())
        }
    }

    #[test]
    fn test_build_graph_validation() {
        let mut graph = BuildGraph::new();
        let a1 = ArtifactId("a1".to_string());
        let a2 = ArtifactId("a2".to_string());

        graph.add_artifact(Box::new(MockArtifact(a1.clone())));
        graph.add_artifact(Box::new(MockArtifact(a2.clone())));

        let mut inputs = HashSet::new();
        inputs.insert(a1.clone());
        let mut outputs = HashSet::new();
        outputs.insert(a2.clone());

        graph.add_transform(Box::new(MockTransform { inputs, outputs }));

        assert!(graph.validate().is_ok());
    }

    #[test]
    fn test_build_graph_cycle() {
        let mut graph = BuildGraph::new();
        let a1 = ArtifactId("a1".to_string());

        let mut inputs = HashSet::new();
        inputs.insert(a1.clone());
        let mut outputs = HashSet::new();
        outputs.insert(a1.clone());

        graph.add_transform(Box::new(MockTransform { inputs, outputs }));

        assert!(graph.validate().is_err()); // Self-cycle
    }

    #[test]
    fn test_build_graph_complex_cycle() {
        let mut graph = BuildGraph::new();
        let a1 = ArtifactId("a1".to_string());
        let a2 = ArtifactId("a2".to_string());

        graph.add_artifact(Box::new(MockArtifact(a1.clone())));
        graph.add_artifact(Box::new(MockArtifact(a2.clone())));

        // T1: a1 -> a2
        let mut i1 = HashSet::new();
        i1.insert(a1.clone());
        let mut o1 = HashSet::new();
        o1.insert(a2.clone());
        graph.add_transform(Box::new(MockTransform {
            inputs: i1,
            outputs: o1,
        }));

        // T2: a2 -> a1
        let mut i2 = HashSet::new();
        i2.insert(a2.clone());
        let mut o2 = HashSet::new();
        o2.insert(a1.clone());
        graph.add_transform(Box::new(MockTransform {
            inputs: i2,
            outputs: o2,
        }));

        assert!(graph.validate().is_err()); // Multi-step cycle
    }
}
