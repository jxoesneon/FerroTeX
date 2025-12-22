use super::*;

#[test]
fn test_delimiter_validator_matching_parens() {
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
fn test_delimiter_validator_mismatched() {
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
fn test_delimiter_validator_unmatched_opening() {
    let delimiters = vec![Delimiter {
        kind: DelimiterKind::LeftParen,
        position: 0,
        is_left_command: false,
    }];

    let mut validator = DelimiterValidator::new();
    validator.validate(&delimiters);
    assert!(validator.has_errors());
}

#[test]
fn test_delimiter_validator_unmatched_closing() {
    let delimiters = vec![Delimiter {
        kind: DelimiterKind::RightParen,
        position: 0,
        is_left_command: false,
    }];

    let mut validator = DelimiterValidator::new();
    validator.validate(&delimiters);
    assert!(validator.has_errors());
}

#[test]
fn test_delimiter_validator_nested() {
    let delimiters = vec![
        Delimiter {
            kind: DelimiterKind::LeftParen,
            position: 0,
            is_left_command: false,
        },
        Delimiter {
            kind: DelimiterKind::LeftBracket,
            position: 2,
            is_left_command: false,
        },
        Delimiter {
            kind: DelimiterKind::RightBracket,
            position: 5,
            is_left_command: false,
        },
        Delimiter {
            kind: DelimiterKind::RightParen,
            position: 7,
            is_left_command: false,
        },
    ];

    let mut validator = DelimiterValidator::new();
    validator.validate(&delimiters);
    assert!(!validator.has_errors());
}

#[test]
fn test_delimiter_validator_nested_mismatch() {
    let delimiters = vec![
        Delimiter {
            kind: DelimiterKind::LeftParen,
            position: 0,
            is_left_command: false,
        },
        Delimiter {
            kind: DelimiterKind::LeftBracket,
            position: 2,
            is_left_command: false,
        },
        Delimiter {
            kind: DelimiterKind::RightParen, // Wrong! Should be RightBracket
            position: 5,
            is_left_command: false,
        },
        Delimiter {
            kind: DelimiterKind::RightBracket,
            position: 7,
            is_left_command: false,
        },
    ];

    let mut validator = DelimiterValidator::new();
    validator.validate(&delimiters);
    assert!(validator.has_errors());
    assert_eq!(validator.errors().len(), 2); // Mismatch + unmatched
}

#[test]
fn test_delimiter_kinds_match() {
    assert!(delimiters_match(&DelimiterKind::LeftParen, &DelimiterKind::RightParen));
    assert!(delimiters_match(&DelimiterKind::LeftBracket, &DelimiterKind::RightBracket));
    assert!(delimiters_match(&DelimiterKind::LeftBrace, &DelimiterKind::RightBrace));
    assert!(delimiters_match(&DelimiterKind::LeftAngle, &DelimiterKind::RightAngle));
    assert!(delimiters_match(&DelimiterKind::LeftFloor, &DelimiterKind::RightFloor));
    assert!(delimiters_match(&DelimiterKind::LeftCeil, &DelimiterKind::RightCeil));
}

#[test]
fn test_delimiter_kinds_dont_match() {
    assert!(!delimiters_match(&DelimiterKind::LeftParen, &DelimiterKind::RightBracket));
    assert!(!delimiters_match(&DelimiterKind::LeftBracket, &DelimiterKind::RightParen));
    assert!(!delimiters_match(&DelimiterKind::LeftBrace, &DelimiterKind::RightAngle));
}

#[test]
fn test_get_expected_args_frac() {
    assert_eq!(get_expected_args("frac"), Some(2));
    assert_eq!(get_expected_args("dfrac"), Some(2));
    assert_eq!(get_expected_args("tfrac"), Some(2));
}

#[test]
fn test_get_expected_args_text() {
    assert_eq!(get_expected_args("text"), Some(1));
    assert_eq!(get_expected_args("mathrm"), Some(1));
    assert_eq!(get_expected_args("mathbf"), Some(1));
}

#[test]
fn test_get_expected_args_unknown() {
    assert_eq!(get_expected_args("unknowncommand"), None);
}

#[test]
fn test_math_error_diagnostic_message() {
    let error = MathError::MismatchedDelimiter {
        left_pos: 0,
        right_pos: 5,
        left_kind: DelimiterKind::LeftParen,
        right_kind: DelimiterKind::RightBracket,
    };
    let msg = error.to_diagnostic_message();
    assert!(msg.contains("Mismatched"));
    assert!(msg.contains("LeftParen"));
    assert!(msg.contains("RightBracket"));
}

#[test]
fn test_math_error_unmatched_opening() {
    let error = MathError::UnmatchedOpening {
        pos: 0,
        kind: DelimiterKind::LeftParen,
    };
    let msg = error.to_diagnostic_message();
    assert!(msg.contains("Unmatched opening"));
}

#[test]
fn test_math_error_incorrect_args() {
    let error = MathError::IncorrectArgumentCount {
        command: "frac".to_string(),
        position: 0,
        expected: 2,
        actual: 1,
    };
    let msg = error.to_diagnostic_message();
    assert!(msg.contains("frac"));
    assert!(msg.contains("2"));
    assert!(msg.contains("1"));
}
