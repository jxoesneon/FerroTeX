use super::{Delimiter, DelimiterKind, MathError, delimiters_match};

/// Validates delimiter matching in LaTeX mathematical expressions.
///
/// ## Algorithm
///
/// This validator uses a **stack-based approach** to verify that delimiters are
/// properly balanced and correctly matched:
///
/// 1. **Opening delimiters** (`(`, `[`, `{`, `\langle`, etc.) are pushed onto a stack
/// 2. **Closing delimiters** (`)`  `]`, `}`, `\rangle`, etc.) pop from the stack
///    and verify the types match
/// 3. At the end, any remaining delimiters on the stack are reported as unmatched
///
/// ## Error Reporting
///
/// The validator collects all errors rather than stopping at the first one,
/// enabling comprehensive diagnostic reporting in the IDE.
///
/// ## Examples
///
/// ### Valid Expression
///
/// ```
/// use ferrotex_core::math_validator::{Delimiter, DelimiterKind, DelimiterValidator};
///
/// let delimiters = vec![
///     Delimiter { kind: DelimiterKind::LeftParen, position: 0, is_left_command: false },
///     Delimiter { kind: DelimiterKind::RightParen, position: 5, is_left_command: false },
/// ];
///
/// let mut validator = DelimiterValidator::new();
/// validator.validate(&delimiters);
/// assert!(!validator.has_errors());
/// ```
///
/// ### Mismatched Delimiters
///
/// ```
/// use ferrotex_core::math_validator::{Delimiter, DelimiterKind, DelimiterValidator};
///
/// let delimiters = vec![
///     Delimiter { kind: DelimiterKind::LeftParen, position: 0, is_left_command: false },
///     Delimiter { kind: DelimiterKind::RightBracket, position: 5, is_left_command: false },
/// ];
///
/// let mut validator = DelimiterValidator::new();
/// validator.validate(&delimiters);
/// assert!(validator.has_errors());
/// assert_eq!(validator.errors().len(), 1);
/// ```
pub struct DelimiterValidator {
    errors: Vec<MathError>,
}

impl DelimiterValidator {
    /// Creates a new validator with an empty error list.
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    /// Validates a sequence of delimiters for proper matching.
    ///
    /// This method processes the delimiter sequence and populates the internal
    /// error list with any validation failures found.
    ///
    /// # Arguments
    ///
    /// * `delimiters` - A slice of delimiters to validate, typically extracted
    ///   from a parsed math expression
    ///
    /// # Algorithm Details
    ///
    /// The validation proceeds as follows:
    ///
    /// 1. For each **opening delimiter**, push it onto the stack
    /// 2. For each **closing delimiter**:
    ///    - If the stack is empty, report [`MathError::UnmatchedClosing`]
    ///    - Otherwise, pop the stack and check if types match
    ///    - If types don't match, report [`MathError::MismatchedDelimiter`]
    /// 3. After processing all delimiters, report [`MathError::UnmatchedOpening`]
    ///    for any remaining stack entries
    pub fn validate(&mut self, delimiters: &[Delimiter]) {
        let mut stack: Vec<&Delimiter> = Vec::new();

        for delim in delimiters {
            match delim.kind {
                DelimiterKind::LeftParen
                | DelimiterKind::LeftBracket
                | DelimiterKind::LeftBrace
                | DelimiterKind::LeftAngle
                | DelimiterKind::LeftFloor
                | DelimiterKind::LeftCeil => {
                    stack.push(delim);
                }
                DelimiterKind::RightParen
                | DelimiterKind::RightBracket
                | DelimiterKind::RightBrace
                | DelimiterKind::RightAngle
                | DelimiterKind::RightFloor
                | DelimiterKind::RightCeil => {
                    if let Some(left) = stack.pop() {
                        if !delimiters_match(&left.kind, &delim.kind) {
                            self.errors.push(MathError::MismatchedDelimiter {
                                left_pos: left.position,
                                right_pos: delim.position,
                                left_kind: left.kind.clone(),
                                right_kind: delim.kind.clone(),
                            });
                        }
                    } else {
                        self.errors.push(MathError::UnmatchedClosing {
                            pos: delim.position,
                            kind: delim.kind.clone(),
                        });
                    }
                }
            }
        }

        // Check for unclosed delimiters
        for left in stack {
            self.errors.push(MathError::UnmatchedOpening {
                pos: left.position,
                kind: left.kind.clone(),
            });
        }
    }

    /// Returns a reference to the collected validation errors.
    ///
    /// # Returns
    ///
    /// A slice of all [`MathError`]s found during validation.
    pub fn errors(&self) -> &[MathError] {
        &self.errors
    }

    /// Checks if any validation errors were found.
    ///
    /// # Returns
    ///
    /// `true` if one or more errors exist, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use ferrotex_core::math_validator::{Delimiter, DelimiterKind, DelimiterValidator};
    ///
    /// let mut validator = DelimiterValidator::new();
    /// let delimiters = vec![
    ///     Delimiter { kind: DelimiterKind::LeftParen, position: 0, is_left_command: false },
    /// ];
    ///
    /// validator.validate(&delimiters);
    /// assert!(validator.has_errors()); // Unmatched opening delimiter
    /// ```
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

impl Default for DelimiterValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matching_delimiters() {
        let delimiters = vec![
            Delimiter {
                kind: DelimiterKind::LeftParen,
                position: 0,
                is_left_command: false,
            },
            Delimiter {
                kind: DelimiterKind::RightParen,
                position: 5,
                is_left_command: false,
            },
        ];

        let mut validator = DelimiterValidator::new();
        validator.validate(&delimiters);
        assert!(!validator.has_errors());
    }

    #[test]
    fn test_mismatched_delimiters() {
        let delimiters = vec![
            Delimiter {
                kind: DelimiterKind::LeftParen,
                position: 0,
                is_left_command: false,
            },
            Delimiter {
                kind: DelimiterKind::RightBracket,
                position: 5,
                is_left_command: false,
            },
        ];

        let mut validator = DelimiterValidator::new();
        validator.validate(&delimiters);
        assert!(validator.has_errors());
        assert_eq!(validator.errors().len(), 1);
    }

    #[test]
    fn test_unmatched_opening() {
        let delimiters = vec![Delimiter {
            kind: DelimiterKind::LeftParen,
            position: 0,
            is_left_command: false,
        }];

        let mut validator = DelimiterValidator::new();
        validator.validate(&delimiters);
        assert!(validator.has_errors());
    }
}
