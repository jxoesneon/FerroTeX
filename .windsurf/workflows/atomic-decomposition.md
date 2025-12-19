---
description: Template for breaking down complex tasks into atomic, chemically precise steps to avoid Cascade Errors.
---

# Atomic Decomposition Workflow

Use this workflow when facing large-scale edits (>300 lines) or complex refactors to ensure high fidelity and avoid hallucinations.

## 1. Objective Definition
- [ ] Clearly state the single, high-level goal (e.g., "Extract Auth logic").
- [ ] Identify the source file(s) and target file(s).

## 2. Atomic Step Breakdown
Break the task into steps that modify *one* logical unit at a time. Use the following template structure:

### Step 1: Create/Scaffold
- **Action**: Create target file `path/to/new_file.ts`.
- **Content**: Add imports and empty function signatures/class shell.
- **Constraint**: Do not implement logic yet.

### Step 2: Move/Implement Logic
- **Action**: Move specific functions (`func1`, `func2`) from Source to Target.
- **Constraint**: Copy exact logic. Do not optimize yet.

### Step 3: Export & Expose
- **Action**: Add `export` keywords in Target.
- **Action**: create/update `index.ts` barrel files if necessary.

### Step 4: Update Consumers
- **Action**: Update import paths in Source file.
- **Constraint**: Verify no other logic in Source is touched.

### Step 5: Verify
- **Action**: Run specific tests for the affected module.

## 3. Execution Rule
- [ ] Execute ONE step at a time.
- [ ] Verify success before moving to the next.
