use ferrotex_log::LogParser;
use std::fs;
use std::path::Path;

#[test]
fn run_golden_tests() {
    let fixtures_dir = Path::new("tests/fixtures");
    if !fixtures_dir.exists() {
        // Skip if no fixtures
        return;
    }

    for entry in fs::read_dir(fixtures_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().map_or(false, |ext| ext == "log") {
            let log_content = fs::read_to_string(&path).expect("Failed to read log");
            let parser = LogParser::new();
            let events = parser.parse(&log_content);

            let json_output =
                serde_json::to_string_pretty(&events).expect("Failed to serialize events");

            let golden_path = path.with_extension("golden.json");

            if std::env::var("UPDATE_GOLDEN").is_ok() {
                fs::write(&golden_path, &json_output).expect("Failed to update golden file");
            } else {
                let expected = fs::read_to_string(&golden_path)
                    .expect("Failed to read golden file (run with UPDATE_GOLDEN=1 to create)");
                // Normalize line endings for comparison if needed
                assert_eq!(
                    json_output.replace("\r\n", "\n"),
                    expected.replace("\r\n", "\n"),
                    "Golden test failed for {:?}",
                    path
                );
            }
        }
    }
}
