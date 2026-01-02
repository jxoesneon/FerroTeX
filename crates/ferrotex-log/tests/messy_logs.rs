use ferrotex_log::parser::LogParser;
use ferrotex_log::ir::EventPayload;

#[test]
fn test_latexmk_noise() {
    let input = include_str!("fixtures/latexmk_noise.txt");
    let parser = LogParser::new();
    let events = parser.parse(input);

    for event in &events {
        println!("{:?}", event);
    }

    // Verify we have expected events
    let file_enters: Vec<_> = events.iter().filter(|e| matches!(e.payload, EventPayload::FileEnter { .. })).collect();
    
    // We expect:
    // 1. (./main.tex
    // 2. (/usr/local/.../article.cls
    // 3. (/usr/local/.../size10.clo)
    // 4. (./setup.tex
    // 5. (./chapter1.tex
    // 6. (./chapter2.tex
    // 7. (./main.aux)
    
    // The "Latexmk: (Info) ..." line should NOT produce a FileEnter event for "Info".
    
    let info_enter = file_enters.iter().find(|e| {
        if let EventPayload::FileEnter { path } = &e.payload {
            path == "Info"
        } else {
            false
        }
    });

    assert!(info_enter.is_none(), "Parser incorrectly interpreted 'Latexmk: (Info)' as a file enter event: {:?}", info_enter);
}
