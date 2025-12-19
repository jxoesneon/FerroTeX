# /ferrotex-release-checklist

Use this Workflow when preparing a **FerroTeX release** (e.g. `v0.x.y`) after implementation work has landed on `main`.

The goal is to enforce release-related parts of `.windsurf/rules`: keep CI green, ensure lint + TeX Live smoke build pass, and treat `docs/` as the canonical source for published documentation.

1. **Gather release context**
   - Ask the user which version they intend to release (e.g. `v0.1.0`).
   - Identify the range of commits since the last tag (e.g. `v0.0.0..HEAD`) and summarize the main themes of changes (features, fixes, docs, infra).

2. **Check current repository health (read-only)**
   - Confirm that:
     - The default branch is `main`.
     - The latest CI run on `main` is green, including:
       - `Lint (Markdown/Prettier)`
       - `Docs build (Jekyll)`
       - `TeX Live smoke build`
       - `Rust Build & Test` (if applicable)
     - The latest Pages deployment for `main` succeeded.
   - Run `cargo test --workspace` locally to ensure no regressions in the Rust codebase.
   - If anything is red, stop and help the user focus on fixing CI **before** proceeding with release steps.

3. **Validate changelog and docs**
   - Open `CHANGELOG.md` and ensure there is an entry for the target version with a clear list of changes.
   - If missing or incomplete, propose a draft changelog entry based on the commit history and existing style.
   - Check `docs/` for any obviously outdated references (e.g. old version numbers in prominent places) and suggest minimal updates where needed.

4. **Plan the release steps (no commands yet)**
   - Propose a concrete, ordered plan tailored to this release, typically including:
     - Update `CHANGELOG.md` (if needed).
     - Ensure version numbers in any relevant files are correct.
     - Commit any remaining release-related edits.
     - Tag the release (e.g. `git tag v0.x.y`).
     - Push commits and tag to GitHub.
     - Let GitHub Actions create the Release and run all workflows.
   - Ask the user to confirm or adjust this plan.

5. **Prepare release commands for the user**
   - Based on the agreed plan, generate the exact git and npm commands the user should run, in order.
   - Clearly mark them as **for the user to run manually**, not for Cascade to execute automatically.
   - Include safety notes:
     - Verify you are on `main` and up to date with `origin/main`.
     - Double-check you are tagging the correct version.
     - **CRITICAL**: Ensure local changes are fully committed and pushed. Releases MUST match `origin/main`.

6. **Post-tag verification checklist**
   - After the user indicates they have pushed the tag, guide them through verifying that on GitHub:
     - The tag exists.
     - A GitHub Release was created (via `Release` workflow).
     - All Release workflow jobs finished successfully.
     - The Pages site still builds and serves correctly.
   - If anything fails, help analyze logs and suggest specific fixes for a follow-up PR.

7. **Record release outcomes**
   - Suggest a short summary the user can add to `CHANGELOG.md` or internal notes describing:
     - Version released.
     - Date.
     - Any notable issues during the release and how they were resolved.

This Workflow should leave the repository in a clean, documented state after a FerroTeX release, ready for the next cycle of implementation work.
