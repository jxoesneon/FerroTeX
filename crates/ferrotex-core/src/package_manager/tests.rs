use super::*;
use std::path::PathBuf;
use crate::package_manager::ctan_db::CTAN_DB;

#[derive(Debug)]
struct MockBackend {
    pub install_result: Result<InstallStatus>,
    pub search_result: Result<Vec<String>>,
}

impl PackageBackend for MockBackend {
    fn install(&self, _package: &str) -> Result<InstallStatus> {
        match &self.install_result {
            Ok(status) => Ok(status.clone()),
            Err(_) => Err(anyhow::anyhow!("Mock error")),
        }
    }

    fn search(&self, _query: &str) -> Result<Vec<String>> {
        match &self.search_result {
            Ok(results) => Ok(results.clone()),
            Err(_) => Err(anyhow::anyhow!("Mock error")),
        }
    }

    fn name(&self) -> &'static str {
        "mock"
    }
}

#[test]
fn test_package_manager_is_available() {
    let pm = PackageManager::with_backend(std::sync::Arc::new(NoOpBackend));
    assert!(!pm.is_available());
    
    let pm2 = PackageManager::with_backend(std::sync::Arc::new(MockBackend { 
        install_result: Ok(InstallStatus { 
            name: "test".into(), 
            state: InstallState::Complete, 
            message: None 
        }),
        search_result: Ok(vec![])
    }));
    assert!(pm2.is_available());
}

#[test]
fn test_ctan_lookup_geometry() {
    let link = CTAN_DB.lookup("geometry.sty");
    assert_eq!(link, Some("geometry"));
    
    // Also test public API
    let link2 = PackageManager::get_ctan_link("geometry.sty");
    assert_eq!(link2, Some("https://ctan.org/pkg/geometry".to_string()));
}

#[test]
fn test_ctan_lookup_nonexistent() {
    let link = CTAN_DB.lookup("nonexistent.sty");
    assert_eq!(link, None);
}

#[test]
fn test_ctan_lookup_case_sensitive() {
    let link = CTAN_DB.lookup("Geometry.sty"); 
    assert_eq!(link, None); 
}

#[test]
fn test_ctan_lookup_tikz() {
    let link = CTAN_DB.lookup("tikz.sty");
    assert_eq!(link, Some("pgf"));
}

#[test]
fn test_ctan_lookup_amsmath() {
    let link = CTAN_DB.lookup("amsmath.sty");
    assert_eq!(link, Some("amsmath"));
}

#[test]
fn test_ctan_db_contains_common_packages() {
    assert!(CTAN_DB.lookup("hyperref.sty").is_some());
    assert!(CTAN_DB.lookup("fancyhdr.sty").is_some());
    assert!(CTAN_DB.lookup("babel.sty").is_some());
}

#[test]
fn test_mock_backend_install_success() {
    let mock = MockBackend {
        install_result: Ok(InstallStatus { 
            name: "test".into(), 
            state: InstallState::Complete, 
            message: None 
        }),
        search_result: Ok(vec![]),
    };
    let pm = PackageManager::with_backend(std::sync::Arc::new(mock));
    let status = pm.install("test").unwrap();
    assert_eq!(status.state, InstallState::Complete);
}

#[test]
fn test_mock_backend_install_failure() {
    let mock = MockBackend {
        install_result: Ok(InstallStatus { 
            name: "test".into(), 
            state: InstallState::Failed, 
            message: Some("Failed".into()) 
        }),
        search_result: Ok(vec![]),
    };
    let pm = PackageManager::with_backend(std::sync::Arc::new(mock));
    let status = pm.install("test").unwrap();
    assert_eq!(status.state, InstallState::Failed);
    assert_eq!(status.message, Some("Failed".into()));
}

#[test]
fn test_mock_backend_search() {
    let mock = MockBackend {
        install_result: Ok(InstallStatus{name: "".into(), state: InstallState::Unknown, message: None}),
        search_result: Ok(vec!["p1".into(), "p2".into()]),
    };
    let pm = PackageManager::with_backend(std::sync::Arc::new(mock));
    let results = pm.search("query").unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0], "p1");
}

#[test]
fn test_real_backends_smoke() {
    let tlmgr = TlmgrBackend::new(PathBuf::from("tlmgr_dummy"));
    let miktex = MiktexBackend::new(PathBuf::from("miktex_dummy"));
    
    // Just verify they can be created. 
    // We can't really call install() without it trying to run a command.
    // But now we can inject a mock executor!
    assert_eq!(tlmgr.name(), "tlmgr");
    assert_eq!(miktex.name(), "miktex");
}

// NEW TESTS USING MOCK EXECUTOR
#[test]
fn test_tlmgr_backend_execution_success() {
    // We create a MockCommandExecutor, which is defined in mod.rs under #[cfg(test)].
    // Since this tests module is a child, we need to access it via super.
    // Actually, MockCommandExecutor is pub inside mod.rs (under cfg test).
    // So `super::MockCommandExecutor` should work.
    let mock = Box::new(super::MockCommandExecutor {
        stdout: "install done".to_string(),
        stderr: "".to_string(),
        status_code: 0,
    });
    
    let backend = TlmgrBackend::with_executor(PathBuf::from("/bin/tlmgr"), mock);
    let status = backend.install("package").unwrap();
    
    assert_eq!(status.state, InstallState::Complete);
    // message is None on success in our impl
}

#[test]
fn test_tlmgr_backend_execution_failure() {
    let mock = Box::new(super::MockCommandExecutor {
        stdout: "".to_string(),
        stderr: "package not found".to_string(),
        status_code: 1,
    });
    
    let backend = TlmgrBackend::with_executor(PathBuf::from("/bin/tlmgr"), mock);
    let status = backend.install("invalid").unwrap();
    
    assert_eq!(status.state, InstallState::Failed);
    assert!(status.message.unwrap().contains("package not found"));
}

#[test]
fn test_tlmgr_search_success() {
    let mock = Box::new(super::MockCommandExecutor {
        stdout: "package1\npackage2".to_string(),
        stderr: "".to_string(),
        status_code: 0,
    });
    let backend = TlmgrBackend::with_executor(PathBuf::from("/bin/tlmgr"), mock);
    let results = backend.search("query").unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0], "package1");
}

#[test]
fn test_tlmgr_search_failure() {
    let mock = Box::new(super::MockCommandExecutor {
        stdout: "".to_string(),
        stderr: "error".to_string(),
        status_code: 1,
    });
    let backend = TlmgrBackend::with_executor(PathBuf::from("/bin/tlmgr"), mock);
    let result = backend.search("query");
    assert!(result.is_err());
}

#[test]
fn test_miktex_backend_execution_success() {
    let mock = Box::new(super::MockCommandExecutor {
        stdout: "".to_string(),
        stderr: "".to_string(),
        status_code: 0,
    });
    
    let backend = MiktexBackend::with_executor(PathBuf::from("/bin/mpm"), mock);
    let status = backend.install("package").unwrap();
    
    assert_eq!(status.state, InstallState::Complete);
}

#[test]
fn test_miktex_backend_execution_failure() {
    let mock = Box::new(super::MockCommandExecutor {
        stdout: "".to_string(),
        stderr: "failed".to_string(),
        status_code: 1,
    });
    
    let backend = MiktexBackend::with_executor(PathBuf::from("/bin/mpm"), mock);
    let status = backend.install("package").unwrap();
    
    assert_eq!(status.state, InstallState::Failed);
}
