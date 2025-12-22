use super::{Delimiter, DelimiterKind, MathError, delimiters_match};

/// Validates bracket and delimiter matching in math expressions
pub struct DelimiterValidator {
    errors: Vec<MathError>,
}

impl DelimiterValidator {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    /// Validate a sequence of delimiters
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

    pub fn errors(&self) -> &[MathError] {
        &self.errors
    }

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
