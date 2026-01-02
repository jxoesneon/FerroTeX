
use crate::LogParser;
use crate::ir::EventPayload;

#[test]
fn test_parse_empty_log() {
    let log = "";
    let parser = LogParser::new();
    let result = parser.parse(log);
    assert!(result.is_empty());
}

#[test]
fn test_parse_simple_error() {
    let log = r#"
! Undefined control sequence.
l.10 \unknowncommand
    "#;
    let parser = LogParser::new();
    let result = parser.parse(log);
    assert!(!result.is_empty());
}

#[test]
fn test_parse_warning() {
    let log = r#"
LaTeX Warning: Reference `fig:unknown' on page 1 undefined on input line 10.
    "#;
    let parser = LogParser::new();
    let result = parser.parse(log);
    assert!(!result.is_empty());
}

#[test]
fn test_parse_overfull_hbox() {
    let log = r#"
Overfull \hbox (10.0pt too wide) in paragraph at lines 5--10
    "#;
    let parser = LogParser::new();
    let result = parser.parse(log);
    assert!(!result.is_empty());
}

#[test]
fn test_parse_underfull_hbox() {
    let log = r#"
Underfull \hbox (badness 10000) in paragraph at lines 5--10
    "#;
    let parser = LogParser::new();
    let result = parser.parse(log);
    assert!(!result.is_empty());
}

#[test]
fn test_parse_file_enter_exit() {
    let log = "(./main.tex)";
    let parser = LogParser::new();
    let result = parser.parse(log);
    // Should have file enter and exit events
    assert!(result.len() >= 2);
}

#[test]
fn test_parser_new() {
    let parser = LogParser::default();
    let result = parser.parse("");
    assert!(result.is_empty());
}

#[test]
fn test_parse_line_reference() {
    let log = r#"
! Error
l.42 some text
    "#;
    let parser = LogParser::new();
    let result = parser.parse(log);
    assert!(!result.is_empty());
}

#[test]
fn test_incremental_parsing() {
    let mut parser = LogParser::new();
    let events1 = parser.update("! Error\n");
    assert!(!events1.is_empty());
    let final_events = parser.finish();
    assert!(events1.len() + final_events.len() > 0);
}

// NEW TESTS
#[test]
fn test_path_spanning_multiple_lines() {
    let mut parser = LogParser::new();
    // Simulate a path wrapped by TeX's line breaking
    let log = "(./long/path/to/\nsome/deeply/nested/\nfile.tex";
    let events = parser.update(log);
    // Flush remaining buffer
    let mut all_events = events;
    all_events.extend(parser.finish());
    
    // Should extract file.tex path joined
    // Note: The logic in extract_path_spanning should handle this joining
    let found = all_events.iter().any(|e| matches!(&e.payload, EventPayload::FileEnter { path } if path.contains("file.tex")));
    assert!(found, "Should have found spanning path");
}

#[test]
fn test_path_interrupted_by_warning() {
    let mut parser = LogParser::new();
    let log = "(./some/broken/\nLaTeX Warning: Reference undefined on input line 5.\nfile.tex";
    let events = parser.update(log);
    
    // Should NOT extract file.tex as part of the previous path
    if let Some(EventPayload::FileEnter { path }) = events.first().map(|e| &e.payload) {
        assert_eq!(path, "./some/broken/");
    } else {
        panic!("First event should be FileEnter");
    }
    
    assert!(events.iter().any(|e| matches!(&e.payload, EventPayload::Warning { .. })));
}

#[test]
fn test_path_interrupted_by_error() {
    let mut parser = LogParser::new();
    let log = "(./some/broken/\n! Undefined control sequence.\n";
    let events = parser.update(log);
    
    if let Some(EventPayload::FileEnter { path }) = events.first().map(|e| &e.payload) {
        assert_eq!(path, "./some/broken/");
    }
    assert!(events.iter().any(|e| matches!(&e.payload, EventPayload::ErrorStart { .. })));
}

#[test]
fn test_garbage_intermixed() {
    let mut parser = LogParser::new();
    let log = "Random text (./file.tex) more text\n(./other.tex\n) closing";
    let events = parser.update(log);
    
    let files: Vec<&String> = events.iter().filter_map(|e| match &e.payload {
        EventPayload::FileEnter { path } => Some(path),
        _ => None,
    }).collect();
    
    assert!(files.contains(&&"./file.tex".to_string()));
    assert!(files.contains(&&"./other.tex".to_string()));
}

#[test]
fn test_parse_ignored_chars() {
    // \r should be ignored/handled
    let log = "Line with \r carriage return";
    let parser = LogParser::new();
    let events = parser.parse(log);
    // Should parse cleanly, maybe no events if no patterns match
    assert!(events.is_empty());
}

#[test]
fn test_parse_complex_error_context() {
    let log = r#"
! LaTeX Error: Something wrong.
See the LaTeX manual or LaTeX Companion for explanation.
Type  H <return>  for immediate help.
 ...                                              
                                                  
l.5 \error
    "#;
    let parser = LogParser::new();
    let events = parser.parse(log);
    assert!(!events.is_empty());
    assert!(events.iter().any(|e| matches!(e.payload, EventPayload::ErrorStart { .. })));
    assert!(events.iter().any(|e| matches!(e.payload, EventPayload::ErrorLineRef { .. })));
}

#[test]
fn test_path_incomplete_peek() {
    // Tests the case where a path extends to the end of a chunk, and the NEXT chunk 
    // does NOT look like a new event. The parser should return "incomplete" (true)
    // and wait for more data.
    let mut parser = LogParser::new();
    let events = parser.update("(path/to/file\nuncorrelated");
    // "uncorrelated" is in the buffer but not processed as line because no newline after it.
    // "(path/to/file" is processed.
    // It sees "\n". peek_line is "uncorrelated".
    // "uncorrelated" does NOT start with valid guard.
    // So extract_path_spanning returns Incomplete.
    // process_buffer breaks loop.
    // No events generated yet.
    assert!(events.is_empty()); 
    // Now we finish
    let final_events = parser.finish();
    // finish() appends " \n". "uncorrelated \n".
    // It reparses. Now we have "(path/to/file\nuncorrelated \n".
    // It should find "path/to/file" joined with "uncorrelated" if generic text?
    // Actually, "uncorrelated" becomes part of the path if not a guard!
    // So we should see one file event: "path/to/fileuncorrelated"
    assert_eq!(final_events.len(), 1); // Only Enter, no Exit without ')'
    // Wait, FileExit is only on ')'.
    // If no ')', we might get FileEnter and it stays open.
    if let Some(EventPayload::FileEnter { path }) = final_events.first().map(|e| &e.payload) {
        assert!(path.contains("path/to/file"));
    }
}

#[test]
fn test_package_warning() {
    let log = "Package hyperref Warning: Token not allowed in a PDF string.";
    let parser = LogParser::new();
    let events = parser.parse(log);
    assert!(!events.is_empty());
    if let EventPayload::Warning { message } = &events[0].payload {
        assert!(message.contains("Token not allowed"));
    } else {
        panic!("Expected Warning");
    }
}
