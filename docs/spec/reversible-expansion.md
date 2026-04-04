# Reversible Expansion Specification

## Status

- **Type:** Normative
- **Stability:** Draft

## Purpose

To support advanced debugging features like "Time-Travel (Step-Backward)" and "Ghost Expansion," FerroTeX implements a **Reversible Abstract Machine**. This specification defines how the machine state is captured at every expansion step to allow non-destructive state transitions.

## The Snapshotting Mechanism

At each step of the macro expansion process, the **Abstract Machine (AM)** captures a complete, immutable snapshot of the **Abstract State**.

### Captured State Elements

Each snapshot contains the following data from the `AbstractState`:

1. **Input Stack (`input_stack`)**: The current sequence of pending tokens or abstract values awaiting processing.
2. **Registers (`registers`)**: A mapping of register names (e.g., `\count0`, `\dimen2`) to their current abstract values.
3. **Expansion Depth (`expansion_depth`)**: The current recursion depth of macro expansion.
4. **Call Stack (`call_stack`)**: The history of nested control sequence expansions to detect and prevent infinite recursion.

### Granularity

Snapshots are taken at the **instruction level** within the expansion engine. This corresponds to:

- Before each macro expansion (`\def`, `\newcommand`).
- Before each conditional branch evaluation (`\if`, `\else`).
- Before each register assignment.
- Before each token consumption.

## Implementation Details

### Incremental Snapshots (Delta Compression)

To avoid excessive memory usage when taking snapshots at every step, the AM uses a **delta-based storage model**.

- **Base State**: Periodically (e.g., every 100 steps), a full state snapshot is stored.
- **Deltas**: Intermediate snapshots only record changes (e.g., "register `\count0` changed from 5 to 6", "popped `\foo` from input_stack").

### Storage in the Reactive Graph

Snapshots are integrated into the **Salsa-based dependency graph** as memoized results of a `step_at(index)` query.

- **Reproducibility**: Given a fixed input and initial state, any step `N` can be perfectly reconstructed.
- **Pruning**: If a preceding change results in an identical state at step `N`, the snapshots for steps `N+1` to `M` remain valid.

## Time-Travel Debugging (Step-Backward)

When the user requests a "Step-Backward" operation in the DAP:

1. The debugger identifies the current step index `I`.
2. The AM retrieves the snapshot at index `I - 1`.
3. The AM restores the `input_stack`, `registers`, and `call_stack` to the values in the snapshot.
4. The editor UI is updated to reflect the restored state.

## Ghost Expansion (Hover Preview)

"Ghost Expansion" leverages reversible expansion by:

1. Creating a **temporary fork** of the Abstract Machine at the user's cursor position.
2. Executing the machine forward from the current snapshot.
3. Discarding the forked state after the preview is generated, ensuring no side effects reach the main analysis state.

## Integration with the TeX Stomach

While expansion occurs in the "mouth" of TeX, the resulting commands are consumed by the "stomach" (e.g., box construction, paragraph breaking). Reversible expansion provides the necessary foundation for reversing stomach operations by ensuring that the input sequence leading to those operations can be precisely replayed or undone.
