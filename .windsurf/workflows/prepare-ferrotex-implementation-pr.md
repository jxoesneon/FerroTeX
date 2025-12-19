# /prepare-ferrotex-implementation-pr

Use this Workflow after you have implemented a FerroTeX change and are getting ready to open a PR.

The goal is to enforce `.windsurf/rules` (trunk-based, CI green, docs as source of truth) and standardize how we prepare implementation PRs.

1. **Understand context and scope**
   - Read `.windsurf/rules` and the repository `README.md` to refresh the governance and quality gates.
   - Ask the user what feature or fix they implemented and which files are expected to change (code, docs, tests, workflows).

2. **Quick repo health + diff scan (read-only)**
   - Without running any commands yet, inspect the workspace for:
     - Changes to `docs/` and `CHANGELOG.md`.
     - Changes to `.github/workflows/` and `scripts/`.
     - Any newly added or removed tests under `tests/`.
   - Summarize the change footprint back to the user and confirm it matches their intent.

3. **Plan validation steps for this PR**
   - Propose a short checklist tailored to this change that includes at least:
     - `npm run lint:yaml`
     - `npm run lint:md`
     - `npm run lint:prettier`
     - `npm run lint:links:internal`
     - TeX smoke build via the CI-equivalent (using `latexmk` on `tests/tex/smoke/main.tex`).
     - `cargo test --workspace` (if Rust changes are involved).
     - `cargo fmt --check` (if Rust changes are involved).
   - Ask the user which of these they want you to prepare commands for, and which they will run themselves.

4. **Prepare, but do not auto-run, local checks**
   - For each selected check, generate the exact shell command(s) the user should run in the project root.
   - Clearly label these as **commands for the user to run**, not commands for Cascade to execute automatically.
   - If any check is expected to be slow or noisy, warn the user.

5. **Review results and suggest fixes**
   - After the user shares the output (or confirms success), interpret the results.
   - If there are failures:
     - Pinpoint the files and lines involved.
     - Propose minimal, targeted edits that resolve the issues while respecting existing style and `.windsurf/rules`.
   - Iterate until the user confirms all local checks are clean or accepts any remaining known limitations.

6. **Docs and tests sanity check**
   - Verify that:
     - User-facing changes have corresponding updates in `docs/` where appropriate.
     - Behavioural changes have at least one test addition or update, or the user explicitly confirms that no test changes are required.
   - If gaps exist, propose concrete doc and test updates and ask whether to implement them.

7. **Prepare the pull request description**
   - Open `.github/pull_request_template.md` and follow its structure.
   - Draft a PR title and description that:
     - Summarizes the change in one concise sentence.
     - Lists key changes as bullet points.
     - Describes which checks were run (lint, TeX smoke, docs build, etc.).
     - Notes any follow-up work or known limitations.
   - Present the draft to the user for editing before they paste it into GitHub.

8. **Final pre-PR checklist**
   - Confirm with the user that:
     - All intended changes are committed.
     - There are no stray debug changes or TODOs accidentally included.
     - CI is expected to be green given the local checks.
   - Output a final, short checklist the user can follow in their terminal:
     - Push branch
     - Open PR against `main`
     - Ensure all required GitHub checks pass

When this Workflow completes, the user should be ready to push their branch and open a high-quality FerroTeX PR that aligns with `.windsurf/rules`.
