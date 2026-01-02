use serde::{Deserialize, Serialize};
use anyhow::Result;

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
        use std::io::{BufRead, BufReader, Read};
        
        let stdin = std::io::stdin();
        let mut reader = BufReader::new(stdin.lock());
        let mut stdout = std::io::stdout();

        loop {
            // 1. Read Headers (Content-Length)
            let mut content_length = 0;
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line)? == 0 {
                    return Ok(()); // EOF
                }
                
                // Trim windows endings
                let line = line.trim();
                
                if line.is_empty() {
                    // Empty line marks end of headers
                    break;
                }
                
                if line.starts_with("Content-Length: ")
                    && let Ok(len) = line["Content-Length: ".len()..].parse::<usize>() {
                    content_length = len;
                }
            }
            
            if content_length == 0 {
                continue; // Should probably error or wait
            }

            // 2. Read Body
            let mut buffer = vec![0u8; content_length];
            reader.read_exact(&mut buffer)?;
            
            let message_str = String::from_utf8_lossy(&buffer);
            // log::debug!("Received: {}", message_str);

            // 3. Parse & Dispatch
            if let Ok(ProtocolMessage::Request { seq, command, arguments }) = serde_json::from_str::<ProtocolMessage>(&message_str) {
                self.handle_request(seq, &command, arguments, &mut stdout)?;
            }
        }
    }
    
    fn handle_request(&mut self, seq: i64, command: &str, args: Option<serde_json::Value>, stdout: &mut impl std::io::Write) -> Result<()> {
        let args = args.unwrap_or(serde_json::Value::Null);
        let result = match command {
            "initialize" => self.adapter.initialize(args),
            "launch" => self.adapter.launch(args).map(|_| serde_json::Value::Null),
            "disconnect" => self.adapter.disconnect().map(|_| serde_json::Value::Null),
            "continue" => self.adapter.continue_execution().map(|_| serde_json::Value::Null),
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
    use std::sync::{Arc, Mutex};
    #[cfg(feature = "tectonic-engine")]
    use std::collections::HashMap;
    #[cfg(feature = "tectonic-engine")]
    use std::sync::mpsc::Sender;

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
            let program = args["program"].as_str().ok_or_else(|| anyhow::anyhow!("Missing 'program' in launch args"))?;
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

        fn step_in(&mut self) -> Result<()> { Ok(()) }

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
    use crate::shim::{Shim, MockShim, EngineCommand};
    use std::sync::mpsc::Sender;
    
    struct MockAdapter {
        shim_tx: Option<Sender<EngineCommand>>,
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
            let shim = MockShim;
            let (tx, _rx) = shim.spawn();
            self.shim_tx = Some(tx);
            // log::info!("Mock Shim Spawned!");
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
        
        fn step_in(&mut self) -> Result<()> { Ok(()) }
        fn scopes(&mut self, _args: serde_json::Value) -> Result<serde_json::Value> {
             Ok(serde_json::json!({ "scopes": [ { "name": "Global", "variablesReference": 1, "expensive": false } ] }))
        }
        fn variables(&mut self, _args: serde_json::Value) -> Result<serde_json::Value> {
             Ok(serde_json::json!({ "variables": [ { "name": "dummy", "value": "0", "variablesReference": 0 } ] }))
        }
        fn disconnect(&mut self) -> Result<()> { 
             if let Some(tx) = &self.shim_tx {
                let _ = tx.send(EngineCommand::Terminate);
            }
            Ok(()) 
        }
    }

    let adapter = MockAdapter { shim_tx: None };
    let mut session = DebugSession::new(adapter);
    session.run_loop()?;
    Ok(())
}

#[cfg(feature = "tectonic-engine")]
pub fn run_tectonic_session() -> Result<()> {
    let adapter = TectonicAdapter::new();
    let mut session = DebugSession::new(adapter);
    session.run_loop()?;
    Ok(())
}
