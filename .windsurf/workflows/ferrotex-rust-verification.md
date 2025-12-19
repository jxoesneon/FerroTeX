# /ferrotex-rust-verification

Use this Workflow to verify the correctness and quality of Rust changes in the FerroTeX repository.

The goal is to ensure all Rust code meets the project's standards for safety, style, and correctness before being merged or released.

1. **Clean Build**
   - Run `cargo clean` (optional, if deep clean needed)
   - Run `cargo build --workspace --all-targets`
   - Confirm no compilation errors or warnings.

2. **Code Formatting**
   - Run `cargo fmt --all -- --check`
   - If this fails, run `cargo fmt --all` to fix, then commit.

3. **Static Analysis (Clippy)**
   - Run `cargo clippy --workspace --all-targets -- -D warnings`
   - Ensure no lints are triggered.

4. **Test Suite**
   - Run `cargo test --workspace`
   - This executes:
     - Unit tests
     - Integration tests
     - Doc tests
     - Golden tests (`tests/golden_tests.rs`)
     - Snapshot tests (`wrapping_tests.rs`)

5. **Fuzzing (Sanity Check)**
   - If parsing logic was touched, run a quick fuzzing cycle:
   - `cargo check` in `fuzz/` directory.
   - Run specific fuzz target (e.g., `cargo fuzz run parser_panic -- -max_total_time=10`) if cargo-fuzz is installed, otherwise build the fuzz target to ensure it compiles.

6. **Documentation**
   - Run `cargo doc --workspace --no-deps`
   - Verify documentation builds without errors.

7. **Final Check**
   - If all steps pass, the Rust codebase is considered stable for this revision.
