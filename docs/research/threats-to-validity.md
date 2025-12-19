# Threats to Validity

## Purpose

Make evaluation limitations explicit and define mitigations.

## Internal Validity

- Mislabeling ground truth
- Dataset bias toward specific engines/distributions

Mitigations:

- labeling guidelines
- double-labeling subset and measuring agreement

## External Validity

- Results may not generalize to all package ecosystems or custom build systems.

Mitigations:

- diverse corpora
- report coverage and failure cases

## Construct Validity

- “Correct file/line mapping” may not fully capture user-perceived usefulness.

Mitigations:

- optional UX study (time-to-fix)
- qualitative analysis of failure cases

## Conclusion Validity

- Performance results sensitive to hardware and OS.

Mitigations:

- publish raw benchmark runs
- include environment metadata
