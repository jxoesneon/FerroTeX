use ferrotex_log::LogParser;
use ferrotex_log::ir::EventPayload;

#[test]
fn test_char_by_char_streaming() {
    let input = r"(./main.tex
LaTeX Warning: Reference `X` on page 1 undefined on input line 10.
)
";
    let mut parser = LogParser::new();
    let mut events = Vec::new();

    // Feed one character at a time
    for c in input.chars() {
        let mut buf = [0; 4];
        let s = c.encode_utf8(&mut buf);
        events.extend(parser.update(s));
    }
    // Finish
    events.extend(parser.finish());

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
