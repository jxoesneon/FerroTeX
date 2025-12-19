use crate::ir::{Confidence, EventPayload, LogEvent, Span};

pub struct LogParser {
    events: Vec<LogEvent>,
    file_stack: Vec<String>,
}

impl LogParser {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            file_stack: Vec::new(),
        }
    }

    pub fn parse(mut self, input: &str) -> Vec<LogEvent> {
        let lines: Vec<&str> = input.lines().collect();
        // Pre-calculate line offsets assuming LF
        let mut line_offsets = Vec::with_capacity(lines.len());
        let mut current_off = 0;
        for line in &lines {
            line_offsets.push(current_off);
            current_off += line.len() + 1;
        }

        let mut line_idx = 0;
        let mut char_idx = 0;

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
                        let (path, consumed_lines, new_char_idx) =
                            self.extract_path_spanning(&lines, line_idx, char_idx + char_len);

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
                        if self.check_warning(
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

        self.events
    }

    fn check_warning(&mut self, text: &str, span_start: usize, span_end: usize) -> bool {
        if text.starts_with("LaTeX Warning:") || text.starts_with("Package") {
            if text.contains("Warning:") {
                self.events.push(LogEvent {
                    span: Span::new(span_start, span_end),
                    confidence: Confidence::default(),
                    payload: EventPayload::Warning {
                        message: text.trim().to_string(),
                    },
                });
                return true;
            }
        }
        if text.starts_with("Overfull \\hbox") || text.starts_with("Underfull \\hbox") {
            self.events.push(LogEvent {
                span: Span::new(span_start, span_end),
                confidence: Confidence::default(),
                payload: EventPayload::Warning {
                    message: text.trim().to_string(),
                },
            });
            return true;
        }
        if text.starts_with("l.") {
            let number_part = &text[2..];
            let digits: String = number_part
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if !digits.is_empty() {
                if let Ok(line_num) = digits.parse::<u32>() {
                    let excerpt = if 2 + digits.len() < text.len() {
                        Some(text[2 + digits.len()..].trim().to_string())
                    } else {
                        None
                    };
                    self.events.push(LogEvent {
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
        }
        false
    }

    fn extract_path_spanning(
        &self,
        lines: &[&str],
        start_line_idx: usize,
        start_char_idx: usize,
    ) -> (String, usize, usize) {
        let mut path = String::new();
        let mut current_line_idx = start_line_idx;
        let mut current_char_idx = start_char_idx;

        loop {
            if current_line_idx >= lines.len() {
                break;
            }
            let line = lines[current_line_idx];
            let remainder = &line[current_char_idx..];

            if let Some(end_idx) = remainder.find(|c: char| c == ')' || c.is_whitespace()) {
                path.push_str(&remainder[..end_idx]);
                return (
                    path,
                    current_line_idx - start_line_idx,
                    current_char_idx + end_idx,
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
                    {
                        // Don't join. Assume path ended at newline.
                        path.push_str(remainder);
                        return (path, current_line_idx - start_line_idx, line.len());
                    }
                }

                path.push_str(remainder);
                current_line_idx += 1;
                current_char_idx = 0;
            }
        }
        (path, current_line_idx - start_line_idx, 0)
    }
}
