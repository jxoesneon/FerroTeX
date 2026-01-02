use crate::ir::{Confidence, EventPayload, LogEvent, Span};

/// A streaming parser for LaTeX logs.
///
/// `LogParser` processes log output incrementally or as a whole, extracting events
/// such as file entry/exit, warnings, and errors. It maintains a stack of open files
/// to track the context of messages.
pub struct LogParser {
    events: Vec<LogEvent>,
    file_stack: Vec<String>,
    buffer: String,
    global_offset: usize,
}

impl Default for LogParser {
    /// Creates a default, empty parser.
    fn default() -> Self {
        Self::new()
    }
}

impl LogParser {
    /// Creates a new, empty `LogParser`.
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            file_stack: Vec::new(),
            buffer: String::new(),
            global_offset: 0,
        }
    }

    /// Appends input to the internal buffer and processes available events.
    ///
    /// # Arguments
    ///
    /// * `input` - A slice of the log file content to append.
    ///
    /// # Returns
    ///
    /// A vector of newly parsed `LogEvent`s.
    pub fn update(&mut self, input: &str) -> Vec<LogEvent> {
        self.buffer.push_str(input);
        self.process_buffer()
    }

    /// Consumes the current parser state and processes any remaining buffer as if it were the end of input.
    ///
    /// # Returns
    ///
    /// All remaining parsed `LogEvent`s, including any from the final buffer flush.
    pub fn finish(mut self) -> Vec<LogEvent> {
        // Ensure any trailing data is processed
        if !self.buffer.is_empty() {
            if !self.buffer.ends_with('\n') {
                self.buffer.push(' '); // Ensure any open token is terminated
                self.buffer.push('\n');
            }
            let mut final_events = self.process_buffer();
            self.events.append(&mut final_events);
        }
        std::mem::take(&mut self.events)
    }

    /// Internal method to process the buffer and drain events.
    fn process_buffer(&mut self) -> Vec<LogEvent> {
        let new_events = Vec::new();
        // We need to keep track of how much of buffer we consumed to advance global_offset
        // and remove consumed part from buffer.

        // This is a naive implementation that still parses line-by-line.
        // Real streaming needs to handle partial lines carefully.
        // For Milestone 1, let's process full lines and leave partial lines in buffer.

        // Find the last newline character to determine the safe processing boundary
        let process_len = if let Some(last_nl) = self.buffer.rfind('\n') {
            last_nl + 1 // Include the newline
        } else {
            return new_events; // No full lines yet
        };

        let chunk = &self.buffer[..process_len];
        let lines: Vec<&str> = chunk.lines().collect();

        let peek_line = if process_len < self.buffer.len() {
            Some(&self.buffer[process_len..])
        } else {
            None
        };

        // ... (Parsing logic adapted from original `parse` method) ...
        // We will adapt the original loop here, but operating on `lines` derived from `chunk`.
        // We need to verify how `extract_path_spanning` behaves with `lines`.

        // Existing logic used `input.lines()`.

        // NOTE: The original logic needs significant adaptation to be re-entrant.
        // For the first step of refactoring, I will paste the original logic body
        // but wrapped to work on the chunk, and then we will refine it to remove processed bytes.

        let mut line_offsets = Vec::with_capacity(lines.len());
        let mut current_off = self.global_offset;
        for line in &lines {
            line_offsets.push(current_off);
            current_off += line.len() + 1; // +1 for assumed \n
        }

        let mut line_idx = 0;
        let mut char_idx = 0;

        // We need to capture events into `new_events` instead of `self.events` for the return value,
        // or just append to `self.events` and return a slice/clone.
        // The original `parse` returned `Vec<LogEvent>`. `update` should probably return new ones.
        // Let's use `self.events` as history if we want, or just ephemeral.
        // The roadmap says "Incremental updates without reparsing".
        // Let's return new events and keep history in `self.events` (or clear it if we don't want history in parser).
        // Actually, typically a parser might keep history or let the caller handle it.
        // Let's clear `self.events` at start of `process_buffer` or use a local vec.
        // But `check_warning` and `extract_path_spanning` might rely on `self` state (file_stack).
        // `check_warning` pushes to `self.events`.

        let start_event_count = self.events.len();

        while line_idx < lines.len() {
            // Check if we exhausted current line
            if char_idx >= lines[line_idx].len() {
                line_idx += 1;
                char_idx = 0;
                continue;
            }

            let line = lines[line_idx];
            let abs_line_start = line_offsets[line_idx];

            let remainder = &line[char_idx..];
            let mut chars = remainder.chars();

            if let Some(c) = chars.next() {
                let char_len = c.len_utf8();
                let current_span_start = abs_line_start + char_idx;

                match c {
                    '(' => {
                        // Extract path, possibly spanning lines
                        let (path, consumed_lines, new_char_idx, incomplete) =
                            Self::extract_path_spanning(
                                &lines,
                                line_idx,
                                char_idx + char_len,
                                peek_line,
                            );

                        if incomplete {
                            // We don't have enough data to finish this path.
                            break;
                        }

                        // VALIDATION: Heuristic to reject non-file text in parentheses
                        // e.g. "Latexmk: (Info) ..." or "TeX Live (preloaded format=...)"
                        // A valid TeX path usually:
                        // - Starts with / or \ or .
                        // - OR looks like a filename with extension (contains a dot)
                        // - We accept likely identifiers if they are long enough and don't contain spaces (extract_path_spanning stops at space)
                        let is_likely_path = path.starts_with('/') 
                                          || path.starts_with('\\') 
                                          || path.starts_with('.')
                                          || (path.contains('.') && !path.ends_with('.'))
                                          || path.contains('/'); // relative path like "subdir/file"
                        
                        // Reject specific false positives seen in logs
                        let is_blacklisted = path == "Info" 
                                           || path == "preloaded"
                                           || path == "TeX"
                                           || path == "con"; // Windows legacy (unlikely in log but good practice)

                        if !is_likely_path || is_blacklisted {
                            // Treat '(' as text
                             char_idx += char_len;
                             continue;
                        }

                        let span_end = if consumed_lines == 0 {
                            abs_line_start + new_char_idx
                        } else {
                            // Calculate end based on new position
                            let final_line_idx = line_idx + consumed_lines;
                            if final_line_idx < lines.len() {
                                line_offsets[final_line_idx] + new_char_idx
                            } else {
                                current_off
                            }
                        };

                        self.file_stack.push(path.clone());
                        self.events.push(LogEvent {
                            span: Span::new(current_span_start, span_end),
                            confidence: Confidence::default(),
                            payload: EventPayload::FileEnter { path },
                        });

                        line_idx += consumed_lines;
                        char_idx = new_char_idx;
                        continue;
                    }
                    ')' => {
                        if let Some(_popped) = self.file_stack.pop() {
                            self.events.push(LogEvent {
                                span: Span::new(current_span_start, current_span_start + 1),
                                confidence: Confidence::default(),
                                payload: EventPayload::FileExit,
                            });
                        } else {
                            self.events.push(LogEvent {
                                span: Span::new(current_span_start, current_span_start + 1),
                                confidence: Confidence(0.5),
                                payload: EventPayload::Info {
                                    message: "Unmatched closing parenthesis".into(),
                                },
                            });
                        }
                        char_idx += char_len;
                    }
                    '!' => {
                        let msg = line[char_idx + char_len..].trim().to_string();
                        self.events.push(LogEvent {
                            span: Span::new(current_span_start, abs_line_start + line.len()),
                            confidence: Confidence::default(),
                            payload: EventPayload::ErrorStart { message: msg },
                        });
                        line_idx += 1;
                        char_idx = 0;
                        continue;
                    }
                    _ => {
                        if Self::check_warning(
                            &mut self.events,
                            &line[char_idx..],
                            current_span_start,
                            abs_line_start + line.len(),
                        ) {
                            line_idx += 1;
                            char_idx = 0;
                            continue;
                        }
                        char_idx += char_len;
                    }
                }
            } else {
                line_idx += 1;
                char_idx = 0;
            }
        }

        // Determine how much we actually processed
        // If we broke early due to incomplete, line_idx tells us where we stopped.
        // Actually, we need to be precise.
        // If we finished the loop successfully, we processed `process_len`.
        // If we broke, we processed up to `line_offsets[line_idx]`.
        // Wait, if we break, `line_idx` points to the line with `(`, which we haven't consumed.

        let consumed_bytes = if line_idx < lines.len() {
            line_offsets[line_idx] - self.global_offset
        } else {
            process_len
        };

        self.global_offset += consumed_bytes;
        self.buffer.drain(..consumed_bytes);

        // Extract new events
        self.events.split_off(start_event_count)
    }

    /// Legacy parse support for backward compatibility.
    ///
    /// This method parses the entire input at once, simulating a full stream update
    /// followed by a finish.
    ///
    /// # Arguments
    ///
    /// * `input` - The full content of the log file.
    ///
    /// # Returns
    ///
    /// A vector of all parsed [`LogEvent`]s.
    pub fn parse(mut self, input: &str) -> Vec<LogEvent> {
        let mut events = self.update(input);
        events.extend(self.finish());
        events
    }

    fn check_warning(
        events: &mut Vec<LogEvent>,
        text: &str,
        span_start: usize,
        span_end: usize,
    ) -> bool {
        if (text.starts_with("LaTeX Warning:") || text.starts_with("Package"))
            && text.contains("Warning:")
        {
            events.push(LogEvent {
                span: Span::new(span_start, span_end),
                confidence: Confidence::default(),
                payload: EventPayload::Warning {
                    message: text.trim().to_string(),
                },
            });
            return true;
        }
        if text.starts_with("Overfull \\hbox") || text.starts_with("Underfull \\hbox") {
            events.push(LogEvent {
                span: Span::new(span_start, span_end),
                confidence: Confidence::default(),
                payload: EventPayload::Warning {
                    message: text.trim().to_string(),
                },
            });
            return true;
        }
        if let Some(number_part) = text.strip_prefix("l.") {
            let digits: String = number_part
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if !digits.is_empty()
                && let Ok(line_num) = digits.parse::<u32>()
            {
                let excerpt = if 2 + digits.len() < text.len() {
                    Some(text[2 + digits.len()..].trim().to_string())
                } else {
                    None
                };
                events.push(LogEvent {
                    span: Span::new(span_start, span_end),
                    confidence: Confidence::default(),
                    payload: EventPayload::ErrorLineRef {
                        line: line_num,
                        source_excerpt: excerpt,
                    },
                });
                return true;
            }
        }
        false
    }

    fn extract_path_spanning(
        lines: &[&str],
        start_line_idx: usize,
        start_char_idx: usize,
        peek_line: Option<&str>,
    ) -> (String, usize, usize, bool) {
        let mut path = String::new();
        let mut current_line_idx = start_line_idx;
        let mut current_char_idx = start_char_idx;

        loop {

            let line = lines[current_line_idx];
            let remainder = &line[current_char_idx..];

            if let Some(end_idx) = remainder.find(|c: char| c == ')' || c.is_whitespace()) {
                path.push_str(&remainder[..end_idx]);
                return (
                    path,
                    current_line_idx - start_line_idx,
                    current_char_idx + end_idx,
                    false,
                );
            } else {
                // Check if we should wrap.
                let next_line_idx = current_line_idx + 1;
                if next_line_idx < lines.len() {
                    let next_line = lines[next_line_idx];
                    // Guarded joining: don't join if next line looks like a new event
                    if next_line.starts_with("LaTeX Warning:")
                        || next_line.starts_with("Package")
                        || next_line.starts_with("!")
                        || next_line.starts_with("(")
                        || next_line.starts_with(")")
                        || next_line.starts_with("Overfull")
                        || next_line.starts_with("Underfull")
                        || next_line.starts_with("LaTeX") // e.g. "LaTeX2e <2020...>"
                        || next_line.starts_with("Document Class:")
                        || next_line.starts_with("L3 programming")
                    {
                        // Don't join. Assume path ended at newline.
                        path.push_str(remainder);
                        return (path, current_line_idx - start_line_idx, line.len(), false);
                    }
                } else {
                    // We are at the last line of the current chunk.
                    // We check peek_line to decide if we should wrap.
                    if let Some(next_line) = peek_line
                        && (next_line.starts_with("LaTeX Warning:")
                            || next_line.starts_with("Package")
                            || next_line.starts_with("!")
                            || next_line.starts_with("(")
                            || next_line.starts_with(")")
                            || next_line.starts_with("Overfull")
                            || next_line.starts_with("Underfull")
                            || next_line.starts_with("LaTeX")
                            || next_line.starts_with("Document Class:")
                            || next_line.starts_with("L3 programming"))
                    {
                        // Don't join.
                        path.push_str(remainder);
                        return (path, current_line_idx - start_line_idx, line.len(), false);
                    }

                    // Otherwise, we can't decide. Incomplete.
                    return (path, current_line_idx - start_line_idx, 0, true);
                }

                path.push_str(remainder);
                current_line_idx += 1;
                current_char_idx = 0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_chunks() {
        let input = "
(main.tex
(package.sty)
LaTeX Warning: Reference `missing' on page 1 undefined on input line 6.
)
";
        // Baseline
        let full_parser = LogParser::new();
        let expected_events = full_parser.parse(input);

        // Streaming with tiny chunks
        let mut stream_parser = LogParser::new();
        let mut amassed_events = Vec::new();
        
        // Chunk size 2 to force many boundaries
        for chunk in input.as_bytes().chunks(2) {
             let s = std::str::from_utf8(chunk).unwrap();
             amassed_events.extend(stream_parser.update(s));
        }
        amassed_events.extend(stream_parser.finish());

        // Compare structure (ignoring spans maybe? No, spans should match if implementation is correct)
        // Spans in streaming might differ if global offset tracking is buggy.
        // Let's compare payload and ordering first.
        
        assert_eq!(expected_events.len(), amassed_events.len());
        for (e1, e2) in expected_events.iter().zip(amassed_events.iter()) {
            assert_eq!(e1.payload, e2.payload);
            // Verify spans if robust
            assert_eq!(e1.span, e2.span, "Span mismatch for payload {:?}", e1.payload);
        }
    }
}
