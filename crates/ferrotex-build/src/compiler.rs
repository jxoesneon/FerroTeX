use crate::{ArtifactId, Transform};
use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Command;

/// A Compiler holds the configuration for executing an external TeX engine.
pub struct Compiler {
    pub engine: String, // e.g., "pdflatex", "xelatex", "tectonic"
    pub output_dir: PathBuf,
    pub extra_args: Vec<String>,
}

impl Compiler {
    pub fn new(engine: &str, output_dir: PathBuf) -> Self {
        Self {
            engine: engine.to_string(),
            output_dir,
            extra_args: Vec::new(),
        }
    }

    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.extra_args = args;
        self
    }
}

/// ShellTransform executes an external shell command as a build step.
pub struct ShellTransform {
    description: String,
    input_ids: HashSet<ArtifactId>,
    output_ids: HashSet<ArtifactId>,
    command: String,
    args: Vec<String>,
    working_dir: Option<PathBuf>,
}

impl ShellTransform {
    pub fn new(
        description: &str,
        input_ids: HashSet<ArtifactId>,
        output_ids: HashSet<ArtifactId>,
        command: &str,
        args: Vec<String>,
    ) -> Self {
        Self {
            description: description.to_string(),
            input_ids,
            output_ids,
            command: command.to_string(),
            args,
            working_dir: None,
        }
    }

    pub fn with_working_dir(mut self, dir: PathBuf) -> Self {
        self.working_dir = Some(dir);
        self
    }
}

impl Transform for ShellTransform {
    fn description(&self) -> String {
        self.description.clone()
    }

    fn inputs(&self) -> HashSet<ArtifactId> {
        self.input_ids.clone()
    }

    fn outputs(&self) -> HashSet<ArtifactId> {
        self.output_ids.clone()
    }

    fn execute(&self) -> Result<(), String> {
        let mut cmd = Command::new(&self.command);
        cmd.args(&self.args);
        
        if let Some(ref dir) = self.working_dir {
            cmd.current_dir(dir);
        }

        let output = cmd.output().map_err(|e| e.to_string())?;
        
        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }
}

/// A PdfLatexTransform is a convenience wrapper for running pdflatex on a .tex file.
pub struct PdfLatexTransform {
    inner: ShellTransform,
}

impl PdfLatexTransform {
    pub fn new(input_tex: ArtifactId, output_pdf: ArtifactId, tex_path: PathBuf, output_dir: PathBuf) -> Self {
        let mut inputs = HashSet::new();
        inputs.insert(input_tex);
        let mut outputs = HashSet::new();
        outputs.insert(output_pdf);

        let args = vec![
            "-interaction=nonstopmode".to_string(),
            format!("-output-directory={}", output_dir.display()),
            tex_path.to_string_lossy().to_string(),
        ];

        let inner = ShellTransform::new(
            "pdflatex compilation",
            inputs,
            outputs,
            "pdflatex",
            args,
        ).with_working_dir(tex_path.parent().unwrap_or(&output_dir).to_path_buf());

        Self { inner }
    }
}

impl Transform for PdfLatexTransform {
    fn description(&self) -> String {
        self.inner.description()
    }
    fn inputs(&self) -> HashSet<ArtifactId> {
        self.inner.inputs()
    }
    fn outputs(&self) -> HashSet<ArtifactId> {
        self.inner.outputs()
    }
    fn execute(&self) -> Result<(), String> {
        self.inner.execute()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ArtifactId;
    use std::collections::HashSet;

    #[test]
    fn test_compiler_config() {
        let out_dir = PathBuf::from("out");
        let compiler = Compiler::new("pdflatex", out_dir.clone())
            .with_args(vec!["-shell-escape".to_string()]);
        
        assert_eq!(compiler.engine, "pdflatex");
        assert_eq!(compiler.output_dir, out_dir);
        assert_eq!(compiler.extra_args[0], "-shell-escape");
    }

    #[test]
    fn test_shell_transform_basic() {
        let mut inputs = HashSet::new();
        inputs.insert(ArtifactId("in".to_string()));
        let mut outputs = HashSet::new();
        outputs.insert(ArtifactId("out".to_string()));
        
        let transform = ShellTransform::new(
            "test echo",
            inputs,
            outputs,
            "echo",
            vec!["hello".to_string()],
        );
        
        assert_eq!(transform.description(), "test echo");
        assert!(transform.execute().is_ok());
    }

    #[test]
    fn test_shell_transform_failure() {
        let transform = ShellTransform::new(
            "failure",
            HashSet::new(),
            HashSet::new(),
            "false",
            vec![],
        );
        assert!(transform.execute().is_err());
    }

    #[test]
    fn test_shell_transform_working_dir() {
        let transform = ShellTransform::new(
            "pwd",
            HashSet::new(),
            HashSet::new(),
            "pwd",
            vec![],
        ).with_working_dir(std::env::current_dir().unwrap());
        assert!(transform.execute().is_ok());
    }

    #[test]
    fn test_pdflatex_transform_delegation() {
        let input = ArtifactId("in.tex".to_string());
        let output = ArtifactId("out.pdf".to_string());
        let tex_path = PathBuf::from("test.tex");
        let out_dir = PathBuf::from("out");
        
        let transform = PdfLatexTransform::new(input, output, tex_path, out_dir);
        assert_eq!(transform.description(), "pdflatex compilation");
        assert_eq!(transform.outputs().len(), 1);
        // This exercises the trait delegation
        let _ = transform.execute();
    }

    #[test]
    fn test_shell_transform_map_err() {
        let transform = ShellTransform::new(
            "invalid",
            HashSet::new(),
            HashSet::new(),
            "/non/existent/command/at/all",
            vec![],
        );
        assert!(transform.execute().is_err());
    }

    #[test]
    fn test_shell_transform_no_dir() {
        let transform = ShellTransform::new(
            "echo",
            HashSet::new(),
            HashSet::new(),
            "echo",
            vec!["ok".to_string()],
        );
        assert!(transform.execute().is_ok());
    }
}
