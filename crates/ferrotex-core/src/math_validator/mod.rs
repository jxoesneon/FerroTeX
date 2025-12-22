use std::collections::HashMap;

/// Represents a mathematical delimiter
#[derive(Debug, Clone, PartialEq)]
pub enum DelimiterKind {
    LeftParen,      // (
    RightParen,     // )
    LeftBracket,    // [
    RightBracket,   // ]
    LeftBrace,      // {
    RightBrace,     // }
    LeftAngle,      // \langle
    RightAngle,     // \rangle
    LeftFloor,      // \lfloor
    RightFloor,     // \rfloor
    LeftCeil,       // \lceil
    RightCeil,      // \rceil
}

/// A paired delimiter in a math expression
#[derive(Debug, Clone)]
pub struct Delimiter {
    pub kind: DelimiterKind,
    pub position: usize,
    pub is_left_command: bool,  // true if using \left or \right
}

/// Expected argument counts for common math commands
pub fn get_expected_args(command: &str) -> Option<usize> {
    let mut map = HashMap::new();
    
    // Fractions and binomials
    map.insert("frac", 2);
    map.insert("dfrac", 2);
    map.insert("tfrac", 2);
    map.insert("cfrac", 2);
    map.insert("binom", 2);
    map.insert("dbinom", 2);
    map.insert("tbinom", 2);
    
    // Roots
    map.insert("sqrt", 1);  // Note: \sqrt can have optional argument
    
    // Text in math
    map.insert("text", 1);
    map.insert("mathrm", 1);
    map.insert("mathbf", 1);
    map.insert("mathit", 1);
    map.insert("mathsf", 1);
    map.insert("mathtt", 1);
    map.insert("mathcal", 1);
    map.insert("mathbb", 1);
    map.insert("mathfrak", 1);
    
    // Operators
    map.insert("overline", 1);
    map.insert("underline", 1);
    map.insert("hat", 1);
    map.insert("tilde", 1);
    map.insert("bar", 1);
    map.insert("vec", 1);
    map.insert("dot", 1);
    map.insert("ddot", 1);
    
    map.get(command).copied()
}

/// Check if delimiters match correctly
pub fn delimiters_match(left: &DelimiterKind, right: &DelimiterKind) -> bool {
    matches!(
        (left, right),
        (DelimiterKind::LeftParen, DelimiterKind::RightParen)
            | (DelimiterKind::LeftBracket, DelimiterKind::RightBracket)
            | (DelimiterKind::LeftBrace, DelimiterKind::RightBrace)
            | (DelimiterKind::LeftAngle, DelimiterKind::RightAngle)
            | (DelimiterKind::LeftFloor, DelimiterKind::RightFloor)
            | (DelimiterKind::LeftCeil, DelimiterKind::RightCeil)
    )
}

/// Math validation error
#[derive(Debug, Clone)]
pub enum MathError {
    MismatchedDelimiter {
        left_pos: usize,
        right_pos: usize,
        left_kind: DelimiterKind,
        right_kind: DelimiterKind,
    },
    UnmatchedOpening {
        pos: usize,
        kind: DelimiterKind,
    },
    UnmatchedClosing {
        pos: usize,
        kind: DelimiterKind,
    },
    IncorrectArgumentCount {
        command: String,
        position: usize,
        expected: usize,
        actual: usize,
    },
}

impl MathError {
    pub fn to_diagnostic_message(&self) -> String {
        match self {
            MathError::MismatchedDelimiter { left_kind, right_kind, .. } => {
                format!("Mismatched delimiters: {:?} paired with {:?}", left_kind, right_kind)
            }
            MathError::UnmatchedOpening { kind, .. } => {
                format!("Unmatched opening delimiter: {:?}", kind)
            }
            MathError::UnmatchedClosing { kind, .. } => {
                format!("Unmatched closing delimiter: {:?}", kind)
            }
            MathError::IncorrectArgumentCount { command, expected, actual, .. } => {
                format!(
                    "Command '\\{}' expects {} argument(s) but got {}",
                    command, expected, actual
                )
            }
        }
    }
}


pub mod delimiter_validator;

#[cfg(test)]
mod tests;

pub use delimiter_validator::DelimiterValidator;
