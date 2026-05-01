use ferrotex_core::package_manager::{InstallState, PackageManager};
use std::env;

#[test]
fn test_package_manager_fallback_to_noop() {
    // Save original PATH
    let original_path = env::var("PATH").unwrap_or_default();

    // Clear PATH to ensure no package managers are found
    env::set_var("PATH", "");

    let pm = PackageManager::new();
    let result = pm.install("some_package");
    assert!(result.is_ok());
    let status = result.unwrap();
    assert_eq!(status.state, InstallState::Unknown);
    assert_eq!(status.message, Some("No package manager found".into()));

    // Restore PATH
    env::set_var("PATH", original_path);
}
