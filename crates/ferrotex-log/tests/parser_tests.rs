use ferrotex_log::LogParser;
use ferrotex_log::ir::EventPayload;

#[test]
fn test_simple_structure() {
    let input = "(./main.tex\nLaTeX Warning: Reference `X` on page 1 undefined on input line 10.\n)";
    let parser = LogParser::new();
    let events = parser.parse(input);

    assert_eq!(events.len(), 3);
    
    // Check FileEnter
    if let EventPayload::FileEnter { path } = &events[0].payload {
        assert_eq!(path, "./main.tex");
    } else {
        panic!("Expected FileEnter");
    }

    // Check Warning
    if let EventPayload::Warning { message } = &events[1].payload {
        assert!(message.contains("Reference `X`"));
    } else {
        panic!("Expected Warning");
    }

    // Check FileExit
    assert!(matches!(events[2].payload, EventPayload::FileExit));
}

#[test]
fn test_error_line_ref() {
    let input = "! Undefined control sequence.\nl.100 \\foo";
    let parser = LogParser::new();
    let events = parser.parse(input);

    assert_eq!(events.len(), 2);

    if let EventPayload::ErrorStart { message } = &events[0].payload {
        assert_eq!(message, "Undefined control sequence.");
    } else {
        panic!("Expected ErrorStart");
    }

    if let EventPayload::ErrorLineRef { line, source_excerpt } = &events[1].payload {
        assert_eq!(*line, 100);
        assert_eq!(source_excerpt.as_deref(), Some("\\foo"));
    } else {
        panic!("Expected ErrorLineRef");
    }
}
