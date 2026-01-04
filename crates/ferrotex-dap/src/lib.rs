use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod shim;

/// Represents a raw DAP message (Request, Response, or Event).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProtocolMessage {
    #[serde(rename = "request")]
    Request {
        seq: i64,
        command: String,
        arguments: Option<serde_json::Value>,
    },
    #[serde(rename = "response")]
    Response {
        seq: i64,
        request_seq: i64,
        success: bool,
        command: String,
        message: Option<String>,
        body: Option<serde_json::Value>,
    },
    #[serde(rename = "event")]
    Event {
        seq: i64,
        event: String,
        body: Option<serde_json::Value>,
    },
}

/// The core trait for a Debug Adapter implementation.
/// Handles the lifecycle of a debug session.
pub trait DebugAdapter {
    /// Called when the client sends the 'initialize' request.
    /// Should return the capabilities of this debug adapter.
    fn initialize(&mut self, args: serde_json::Value) -> Result<serde_json::Value>;

    /// Called when the client sends 'launch' or 'attach'.
    /// Starts the debuggee process.
    fn launch(&mut self, args: serde_json::Value) -> Result<()>;

    /// Called when the client requests 'continue'.
    fn continue_execution(&mut self) -> Result<()>;

    /// Called when the client requests 'next' (step over).
    fn next(&mut self) -> Result<()>;

    /// Called when the client requests 'stepIn'.
    fn step_in(&mut self) -> Result<()>;

    /// Called when the client requests 'scopes'.
    fn scopes(&mut self, args: serde_json::Value) -> Result<serde_json::Value>;

    /// Called when the client requests 'variables'.
    fn variables(&mut self, args: serde_json::Value) -> Result<serde_json::Value>;

    /// Called to disconnect/terminate the session.
    fn disconnect(&mut self) -> Result<()>;
}

/// A generic session handler that wraps a specific Adapter implementation
/// and handles the raw protocol loop (reading stdin, writing stdout).
pub struct DebugSession<A: DebugAdapter> {
    adapter: A,
    seq: i64,
}

impl<A: DebugAdapter> DebugSession<A> {
    pub fn new(adapter: A) -> Self {
        Self { adapter, seq: 1 }
    }

    /// Starts the message loop, reading from stdin and writing to stdout.
    /// This is the entry point for the DAP server.
    pub fn run_loop(&mut self) -> Result<()> {
        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();
        self.run_session(&mut stdin.lock(), &mut stdout)
    }

    /// Internal logic for the DAP session, allowing mocking of I/O.
    pub fn run_session(
        &mut self,
        reader: &mut impl std::io::BufRead,
        stdout: &mut impl std::io::Write,
    ) -> Result<()> {
        loop {
            // 1. Read Headers (Content-Length)
            let mut content_length = 0;
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line)? == 0 {
                    return Ok(()); // EOF
                }

                // Trim
                let line = line.trim();

                if line.is_empty() {
                    // Empty line marks end of headers
                    break;
                }

                if line.to_lowercase().starts_with("content-length: ")
                    && let Ok(len) = line["content-length: ".len()..].parse::<usize>()
                {
                    content_length = len;
                }
            }

            if content_length == 0 {
                continue;
            }

            // 2. Read Body
            let mut buffer = vec![0u8; content_length];
            reader.read_exact(&mut buffer)?;

            let message_str = String::from_utf8_lossy(&buffer);

            // 3. Parse & Dispatch
            if let Ok(msg) = serde_json::from_str::<ProtocolMessage>(&message_str)
                && let ProtocolMessage::Request {
                    seq,
                    command,
                    arguments,
                } = msg
            {
                self.handle_request(seq, &command, arguments, stdout)?;
            }
        }
    }

    fn handle_request(
        &mut self,
        seq: i64,
        command: &str,
        args: Option<serde_json::Value>,
        stdout: &mut impl std::io::Write,
    ) -> Result<()> {
        let args = args.unwrap_or(serde_json::Value::Null);
        let result = match command {
            "initialize" => self.adapter.initialize(args),
            "launch" => self.adapter.launch(args).map(|_| serde_json::Value::Null),
            "disconnect" => self.adapter.disconnect().map(|_| serde_json::Value::Null),
            "continue" => self
                .adapter
                .continue_execution()
                .map(|_| serde_json::Value::Null),
            "next" => self.adapter.next().map(|_| serde_json::Value::Null),
            "stepIn" => self.adapter.step_in().map(|_| serde_json::Value::Null),
            "scopes" => self.adapter.scopes(args),
            "variables" => self.adapter.variables(args),
            _ => Ok(serde_json::json!({})),
        };

        let (success, body, message) = match result {
            Ok(val) => (true, Some(val), None),
            Err(e) => (false, None, Some(e.to_string())),
        };

        let response = ProtocolMessage::Response {
            seq: self.next_seq(),
            request_seq: seq,
            success,
            command: command.to_string(),
            message,
            body,
        };

        let resp_json = serde_json::to_string(&response)?;
        let resp_body = format!("Content-Length: {}\r\n\r\n{}", resp_json.len(), resp_json);
        stdout.write_all(resp_body.as_bytes())?;
        stdout.flush()?;

        Ok(())
    }

    fn next_seq(&mut self) -> i64 {
        self.seq += 1;
        self.seq
    }
}

#[cfg(feature = "tectonic-engine")]
use crate::shim::{EngineCommand, EngineEvent};
#[cfg(feature = "tectonic-engine")]
use std::collections::HashMap;
#[cfg(feature = "tectonic-engine")]
use std::sync::mpsc::Sender;
#[cfg(feature = "tectonic-engine")]
use std::sync::{Arc, Mutex};

#[cfg(feature = "tectonic-engine")]
pub struct TectonicAdapter {
    shim_tx: Option<Sender<EngineCommand>>,
    shadow_vars: Arc<Mutex<HashMap<String, String>>>,
}

#[cfg(feature = "tectonic-engine")]
impl TectonicAdapter {
    pub fn new() -> Self {
        Self {
            shim_tx: None,
            shadow_vars: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[cfg(feature = "tectonic-engine")]
impl DebugAdapter for TectonicAdapter {
    fn initialize(&mut self, _args: serde_json::Value) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "supportsConfigurationDoneRequest": true,
            "supportsVariableType": true,
            "supportsVariablePaging": false,
        }))
    }

    fn launch(&mut self, args: serde_json::Value) -> Result<()> {
        use crate::shim::TectonicShim;
        let program = args["program"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'program' in launch args"))?;
        let shim = TectonicShim::new(std::path::PathBuf::from(program));
        let (tx, rx) = shim.spawn();
        self.shim_tx = Some(tx);

        let vars = self.shadow_vars.clone();
        // Thread to handle events from the engine
        std::thread::spawn(move || {
            let mut stdout = std::io::stdout();
            while let Ok(event) = rx.recv() {
                match event {
                    EngineEvent::Stopped { reason, location } => {
                        let msg = serde_json::json!({
                            "type": "event",
                            "seq": 0, // Injected by session usually, but here we're async
                            "event": "stopped",
                            "body": {
                                "reason": reason,
                                "threadId": 1,
                                "allThreadsStopped": true,
                                "text": location
                            }
                        });
                        send_raw_dap(&msg, &mut stdout).unwrap();
                    }
                    EngineEvent::Output(text) => {
                        let msg = serde_json::json!({
                            "type": "event",
                            "event": "output",
                            "body": { "output": text }
                        });
                        send_raw_dap(&msg, &mut stdout).unwrap();
                    }
                    EngineEvent::VariablesUpdated(new_vars) => {
                        let mut v = vars.lock().unwrap();
                        *v = new_vars;
                    }
                    EngineEvent::Terminated => {
                        let msg = serde_json::json!({ "type": "event", "event": "terminated" });
                        send_raw_dap(&msg, &mut stdout).unwrap();
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    fn continue_execution(&mut self) -> Result<()> {
        if let Some(tx) = &self.shim_tx {
            tx.send(EngineCommand::Continue)?;
        }
        Ok(())
    }

    fn next(&mut self) -> Result<()> {
        if let Some(tx) = &self.shim_tx {
            tx.send(EngineCommand::Step)?;
        }
        Ok(())
    }

    fn step_in(&mut self) -> Result<()> {
        Ok(())
    }

    fn scopes(&mut self, _args: serde_json::Value) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "scopes": [
                { "name": "Registers", "variablesReference": 1, "expensive": false },
            ]
        }))
    }

    fn variables(&mut self, _args: serde_json::Value) -> Result<serde_json::Value> {
        let vars = self.shadow_vars.lock().unwrap();
        let mut dap_vars = Vec::new();
        for (name, value) in vars.iter() {
            dap_vars.push(serde_json::json!({
                "name": name,
                "value": value,
                "variablesReference": 0
            }));
        }
        Ok(serde_json::json!({ "variables": dap_vars }))
    }

    fn disconnect(&mut self) -> Result<()> {
        if let Some(tx) = &self.shim_tx {
            let _ = tx.send(EngineCommand::Terminate);
        }
        Ok(())
    }
}

#[cfg(feature = "tectonic-engine")]
fn send_raw_dap(msg: &serde_json::Value, stdout: &mut impl std::io::Write) -> Result<()> {
    let resp_json = serde_json::to_string(msg)?;
    let resp_body = format!("Content-Length: {}\r\n\r\n{}", resp_json.len(), resp_json);
    stdout.write_all(resp_body.as_bytes())?;
    stdout.flush()?;
    Ok(())
}

pub fn run_mock_session() -> Result<()> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    run_mock_session_with_io(&mut stdin.lock(), &mut stdout)
}

struct MockAdapter {
    shim_tx: Option<std::sync::mpsc::Sender<crate::shim::EngineCommand>>,
}

impl DebugAdapter for MockAdapter {
    fn initialize(&mut self, _args: serde_json::Value) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "supportsConfigurationDoneRequest": true,
            "supportsFunctionBreakpoints": false,
            "supportsConditionalBreakpoints": false,
        }))
    }

    fn launch(&mut self, _args: serde_json::Value) -> Result<()> {
        use crate::shim::Shim;
        let shim = crate::shim::MockShim;
        let (tx, _rx) = shim.spawn();
        self.shim_tx = Some(tx);
        Ok(())
    }

    fn continue_execution(&mut self) -> Result<()> {
        if let Some(tx) = &self.shim_tx {
            tx.send(crate::shim::EngineCommand::Continue)?;
        }
        Ok(())
    }

    fn next(&mut self) -> Result<()> {
        if let Some(tx) = &self.shim_tx {
            tx.send(crate::shim::EngineCommand::Step)?;
        }
        Ok(())
    }

    fn step_in(&mut self) -> Result<()> {
        Ok(())
    }
    fn scopes(&mut self, _args: serde_json::Value) -> Result<serde_json::Value> {
        Ok(
            serde_json::json!({ "scopes": [ { "name": "Global", "variablesReference": 1, "expensive": false } ] }),
        )
    }
    fn variables(&mut self, _args: serde_json::Value) -> Result<serde_json::Value> {
        Ok(
            serde_json::json!({ "variables": [ { "name": "dummy", "value": "0", "variablesReference": 0 } ] }),
        )
    }
    fn disconnect(&mut self) -> Result<()> {
        if let Some(tx) = &self.shim_tx {
            let _ = tx.send(crate::shim::EngineCommand::Terminate);
        }
        Ok(())
    }
}

pub fn run_mock_session_with_io(
    reader: &mut impl std::io::BufRead,
    writer: &mut impl std::io::Write,
) -> Result<()> {
    let adapter = MockAdapter { shim_tx: None };
    let mut session = DebugSession::new(adapter);
    session.run_session(reader, writer)?;
    Ok(())
}

#[cfg(feature = "tectonic-engine")]
pub fn run_tectonic_session() -> Result<()> {
    let adapter = TectonicAdapter::new();
    let mut session = DebugSession::new(adapter);
    session.run_loop()?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_protocol_message_serialization() {
        let msg = ProtocolMessage::Request {
            seq: 1,
            command: "initialize".to_string(),
            arguments: Some(json!({"adapterID": "ferrotex"})),
        };
        let js = serde_json::to_string(&msg).unwrap();
        assert!(js.contains("\"type\":\"request\""));
        assert!(js.contains("\"command\":\"initialize\""));

        let back: ProtocolMessage = serde_json::from_str(&js).unwrap();
        match back {
            ProtocolMessage::Request { seq, .. } => assert_eq!(seq, 1),
            _ => panic!("Wrong type"),
        }
    }

    struct SimpleAdapter;
    impl DebugAdapter for SimpleAdapter {
        fn initialize(&mut self, _args: serde_json::Value) -> Result<serde_json::Value> {
            Ok(json!({"ok": true}))
        }
        fn launch(&mut self, _args: serde_json::Value) -> Result<()> {
            Ok(())
        }
        fn continue_execution(&mut self) -> Result<()> {
            Ok(())
        }
        fn next(&mut self) -> Result<()> {
            Ok(())
        }
        fn step_in(&mut self) -> Result<()> {
            Ok(())
        }
        fn scopes(&mut self, _args: serde_json::Value) -> Result<serde_json::Value> {
            Ok(json!({}))
        }
        fn variables(&mut self, _args: serde_json::Value) -> Result<serde_json::Value> {
            Ok(json!({}))
        }
        fn disconnect(&mut self) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_session_handle_request() {
        let mut session = DebugSession::new(SimpleAdapter);
        let mut stdout = Vec::new();

        session
            .handle_request(1, "initialize", None, &mut stdout)
            .unwrap();

        let out_str = String::from_utf8(stdout).unwrap();
        assert!(out_str.contains("Content-Length:"));
        assert!(out_str.contains("\"success\":true"));
        assert!(out_str.contains("\"ok\":true"));
    }

    #[test]
    fn test_session_run_loop_mock() {
        let mut session = DebugSession::new(SimpleAdapter);
        let body = json!({"type":"request","seq":1,"command":"next"}).to_string();
        let input_data = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);
        let mut reader = std::io::Cursor::new(input_data);
        let mut stdout = Vec::new();

        session.run_session(&mut reader, &mut stdout).unwrap();

        let out_str = String::from_utf8(stdout).unwrap();
        assert!(out_str.contains("\"command\":\"next\""));
        assert!(out_str.contains("\"success\":true"));
    }

    #[test]
    fn test_adapter_error() {
        struct FailingAdapter;
        impl DebugAdapter for FailingAdapter {
            fn initialize(&mut self, _args: serde_json::Value) -> Result<serde_json::Value> {
                Err(anyhow::anyhow!("fail"))
            }
            fn launch(&mut self, _args: serde_json::Value) -> Result<()> {
                Ok(())
            }
            fn continue_execution(&mut self) -> Result<()> {
                Ok(())
            }
            fn next(&mut self) -> Result<()> {
                Ok(())
            }
            fn step_in(&mut self) -> Result<()> {
                Ok(())
            }
            fn scopes(&mut self, _args: serde_json::Value) -> Result<serde_json::Value> {
                Ok(json!({}))
            }
            fn variables(&mut self, _args: serde_json::Value) -> Result<serde_json::Value> {
                Ok(json!({}))
            }
            fn disconnect(&mut self) -> Result<()> {
                Ok(())
            }
        }

        let mut session = DebugSession::new(FailingAdapter);
        let mut stdout = Vec::new();
        session
            .handle_request(1, "initialize", None, &mut stdout)
            .unwrap();
        let out_str = String::from_utf8(stdout).unwrap();
        assert!(out_str.contains("\"success\":false"));
        assert!(out_str.contains("\"message\":\"fail\""));
    }

    #[test]
    fn test_run_mock_session_with_io() {
        let commands = vec![
            json!({"type":"request","seq":1,"command":"initialize","arguments":{"adapterID":"test"}}),
            json!({"type":"request","seq":2,"command":"launch","arguments":{"program":"test"}}),
            json!({"type":"request","seq":3,"command":"continue"}),
            json!({"type":"request","seq":4,"command":"next"}),
            json!({"type":"request","seq":5,"command":"stepIn"}),
            json!({"type":"request","seq":6,"command":"scopes","arguments":{"frameId":1}}),
            json!({"type":"request","seq":7,"command":"variables","arguments":{"variablesReference":1}}),
            json!({"type":"request","seq":8,"command":"disconnect"}),
        ];

        let mut input_data = String::new();
        for cmd in commands {
            let body = cmd.to_string();
            input_data.push_str(&format!("Content-Length: {}\r\n\r\n{}", body.len(), body));
        }

        let mut reader = std::io::Cursor::new(input_data);
        let mut stdout = Vec::new();

        run_mock_session_with_io(&mut reader, &mut stdout).unwrap();

        let out_str = String::from_utf8(stdout).unwrap();
        assert!(out_str.contains("\"command\":\"initialize\""));
        assert!(out_str.contains("\"command\":\"launch\""));
        assert!(out_str.contains("\"command\":\"continue\""));
        assert!(out_str.contains("\"command\":\"next\""));
        assert!(out_str.contains("\"command\":\"disconnect\""));
    }

    #[test]
    fn test_session_eof() {
        let mut session = DebugSession::new(SimpleAdapter);
        let mut reader = std::io::Cursor::new("");
        let mut stdout = Vec::new();
        assert!(session.run_session(&mut reader, &mut stdout).is_ok());
    }
}
