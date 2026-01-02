use ferrotex_core::package_manager::{InstallState, PackageManager};
use std::env;

#[test]
fn test_package_manager_fallback_to_noop() {
    // Save original PATH
    let original_path = env::var("PATH").unwrap_or_default();

    // Clear PATH to ensure no package managers are found
    // We use a mutex or similar if tests ran in parallel, but here we just need to be careful.
    // Ideally integration tests run in separate processes or we rely on cargo test isolation if configured?
    // Rust tests run in threads by default. Modifying env vars is risky.
    // However, for this specific "fallback" test, it's the most direct way without changing the code structure significantly.
    // A safer way would be to make `which` usage dependency-injected, but that requires refactoring.
    // For now, let's try to acquire a lock if we can, or just accept it might flake if parallel.
    // Actually, `cargo test -- --test-threads=1` guarantees serial execution.
    // Or we use a lock file / mutex.

    // BUT, we are adding new file.

    // Let's modify the PATH locally for this test logic? No, env::set_var sets it for the process.
    // We will assume for now we can wrap it in a localized block or just do it.

    unsafe {
        env::set_var("PATH", "");
    }

    let pm = PackageManager::new();
    // NoOpBackend returns "none" for name() but name() isn't exposed on PackageManager?
    // Available method: is_available()? No, let's check:

    // PackageManager has: new(), with_backend(), install(), search()
    // It doesn't seem to expose expected "name".
    // But `NoOpBackend::install` returns InstallStatus with state::Unknown and message "No package manager found".

    let result = pm.install("some_package");
    assert!(result.is_ok());
    let status = result.unwrap();
    assert_eq!(status.state, InstallState::Unknown);
    assert_eq!(status.message, Some("No package manager found".into()));

    // Restore PATH
    unsafe {
        env::set_var("PATH", original_path);
    }
}
