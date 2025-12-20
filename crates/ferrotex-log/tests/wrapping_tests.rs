use ferrotex_log::ir::EventPayload;
use ferrotex_log::LogParser;

#[test]
fn test_wrapped_filename() {
    // TeX often wraps lines at 79 characters.
    // This input simulates a filename split across lines.
    // Note: The newline is significant in the raw string.
    let input = "(./some/very/long/path/to/a/file/that/gets/wrapp\ned/here.tex\n)";

    let parser = LogParser::new();
    let events = parser.parse(input);

    // We expect 2 events: FileEnter and FileExit (plus maybe Info if recovery is noisy, but ideally clean)
    // The path should be joined.

    // Find FileEnter
    let file_enter = events
        .iter()
        .find(|e| matches!(e.payload, EventPayload::FileEnter { .. }));
    assert!(file_enter.is_some(), "Should find FileEnter event");

    if let EventPayload::FileEnter { path } = &file_enter.unwrap().payload {
        assert_eq!(
            path,
            "./some/very/long/path/to/a/file/that/gets/wrapped/here.tex"
        );
    } else {
        panic!("Payload match failed");
    }
}
