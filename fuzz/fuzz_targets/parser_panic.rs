#![no_main]
use ferrotex_log::LogParser;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Basic fuzzing for panic freedom.
    // The parser expects &str, so we convert.
    // We use lossy conversion to maximize coverage of inputs that are "almost" text.
    let s = String::from_utf8_lossy(data);
    let parser = LogParser::new();
    let _ = parser.parse(&s);
});
