---
description: Large-scale refactoring using the Memento Pattern to maintain context across sessions.
---

# Memento Pattern Refactoring Workflow

This workflow implements the "Memento Pattern" for large-scale refactoring, ensuring context persistence across context window limits.

## 1. State Externalization

- [ ] Create a `refactor_plan.md` file in the root of the workspace if it doesn't exist.
- [ ] Define the high-level goal and specific milestones in `refactor_plan.md`.
- [ ] List the current state of the system and known constraints.

## 2. Initialization

- [ ] Read `refactor_plan.md` to ground the current session.
- [ ] Confirm understanding of the next immediate milestone.

## 3. Atomic Execution (Cycle)

- [ ] Pick **ONE** atomic task from the plan (e.g., "Extract function X to file Y").
- [ ] **Critical:** Do not attempt multiple milestones in one step.
- [ ] Execute the change using `write_to_file` or `edit`.
- [ ] Verify the change (run tests/lint).

## 4. Checkpointing

- [ ] Update `refactor_plan.md` immediately after verification.
  - Mark the task as `[x]`.
  - Add notes about any unexpected discoveries or deviations.
  - Update the "Current State" section.

## 5. Context Flush (If needed)

- [ ] If performance degrades or the context window fills:
  - Ensure `refactor_plan.md` is up to date.
  - Commit current changes to git.
  - **STOP** and instruct the user to restart the session, pointing the new session to `refactor_plan.md`.
