use std::sync::mpsc::{Sender, Receiver};

#[derive(Debug, Clone)]
pub enum EngineEvent {
    /// The engine has paused at a breakpoint or step.
    Stopped { reason: String, location: String },
    /// The engine has terminated.
    Terminated,
    /// Output from the engine (stdout).
    Output(String),
    /// Variables have been updated.
    VariablesUpdated(std::collections::HashMap<String, String>),
}

#[derive(Debug, Clone)]
pub enum EngineCommand {
    Continue,
    Step,
    Pause,
    Terminate,
}

/// A shim wraps a TeX engine (real or mock) and provides channel-based control.
pub trait Shim {
    /// Starts the engine in a background thread.
    /// Returns channels for sending commands and receiving events.
    fn spawn(&self) -> (Sender<EngineCommand>, Receiver<EngineEvent>);
}

pub struct MockShim;

impl Shim for MockShim {
    fn spawn(&self) -> (Sender<EngineCommand>, Receiver<EngineEvent>) {
        let (cmd_tx, cmd_rx) = std::sync::mpsc::channel();
        let (event_tx, event_rx) = std::sync::mpsc::channel();
        
        std::thread::spawn(move || {
            let mut steps = 0;
            loop {
                // Wait for command
                match cmd_rx.recv() {
                    Ok(EngineCommand::Continue) => {
                        // Simulate running for a bit then stopping
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        let _ = event_tx.send(EngineEvent::Output(format!("Processing chunk {}\n", steps)));
                        steps += 1;
                        if steps > 5 {
                            let _ = event_tx.send(EngineEvent::Terminated);
                            break;
                        }
                    }
                    Ok(EngineCommand::Step) => {
                        // Step one "instruction"
                        let _ = event_tx.send(EngineEvent::Output(format!("Step {}\n", steps)));
                        steps += 1;
                        let _ = event_tx.send(EngineEvent::Stopped { 
                            reason: "step".to_string(), 
                            location: format!("line {}", steps) 
                        });
                    }
                    Ok(EngineCommand::Terminate) => break,
                    _ => break,
                }
            }
        });
        
        (cmd_tx, event_rx)
    }
}

#[cfg(feature = "tectonic-engine")]
mod stepping_io {
    use sha2::{Sha256, Digest};
    use std::collections::HashMap;
    use tectonic_io_base::{IoProvider, OpenResult, InputHandle, OutputHandle, IoStatus};
    use std::sync::{Arc, Mutex, Condvar};
    use crate::shim::{EngineEvent};

    pub struct SteppingIoProvider<T: IoProvider> {
        inner: T,
        event_tx: std::sync::mpsc::Sender<EngineEvent>,
        /// Shared state for blocking/unblocking
        control: Arc<(Mutex<bool>, Condvar)>,
        /// Tracked file hashes (path -> sha256)
        hashes: Arc<Mutex<HashMap<String, String>>>,
        /// Name of the primary file to inject traces into
        primary_file: Option<String>,
    }

    impl<T: IoProvider> SteppingIoProvider<T> {
        pub fn new(
            inner: T, 
            event_tx: std::sync::mpsc::Sender<EngineEvent>, 
            control: Arc<(Mutex<bool>, Condvar)>,
            hashes: Arc<Mutex<HashMap<String, String>>>,
            primary_file: Option<String>,
        ) -> Self {
            Self { inner, event_tx, control, hashes, primary_file }
        }

        fn wait_for_continue(&self, name: &str) {
            // 1. Notify DAP that we stopped on a file
            let _ = self.event_tx.send(EngineEvent::Stopped {
                reason: "file_access".to_string(),
                location: name.to_string(),
            });

            // 2. Block until control set to true
            let (lock, cvar) = &*self.control;
            let mut started = lock.lock().unwrap();
            *started = false; // Reset for next step
            
            while !*started {
                started = cvar.wait(started).unwrap();
            }
        }
    }

    impl<T: IoProvider> IoProvider for SteppingIoProvider<T> {
        fn open_input(&mut self, name: &str) -> OpenResult<InputHandle> {
            // Only stop on "interesting" files (not core formats)
            if name.ends_with(".tex") || name.ends_with(".sty") || name.ends_with(".cls") {
                let _ = self.event_tx.send(EngineEvent::Output(format!("📖 Opening: {}\n", name)));
                
                // Track hash
                if let Ok(data) = std::fs::read(name) {
                    let mut hasher = Sha256::new();
                    hasher.update(&data);
                    let hash = hex::encode(hasher.finalize());
                    self.hashes.lock().unwrap().insert(name.to_string(), hash);

                    // If this is the primary file, inject tracing flags
                    if self.primary_file.as_deref() == Some(name) {
                        let mut augmented = b"\\tracingassigns=1\\tracingonline=1\\tracingmacros=1\n".to_vec();
                        augmented.extend_from_slice(&data);
                        
                        self.wait_for_continue(name);
                        return Ok(InputHandle::new_memory_backed(augmented));
                    }
                }
                
                self.wait_for_continue(name);
            }
            self.inner.open_input(name)
        }

        fn open_output(&mut self, name: &str, status: IoStatus) -> OpenResult<OutputHandle> {
            self.inner.open_output(name, status)
        }
    }
}

/// A shim that uses the real Tectonic engine (requires `tectonic-engine` feature).
/// 
/// This implementation provides pass-level stepping (TeX pass, bibtex pass, etc.)
/// and forwards Tectonic status messages as DAP engine events.
#[cfg(feature = "tectonic-engine")]
pub struct TectonicShim {
    pub tex_path: std::path::PathBuf,
}

#[cfg(feature = "tectonic-engine")]
impl TectonicShim {
    pub fn new(tex_path: std::path::PathBuf) -> Self {
        Self { tex_path }
    }
}

#[cfg(feature = "tectonic-engine")]
impl Shim for TectonicShim {
    fn spawn(&self) -> (Sender<EngineCommand>, Receiver<EngineEvent>) {
        use tectonic::config::PersistentConfig;
        use tectonic::driver::{ProcessingSessionBuilder, OutputFormat, PassSetting};
        use tectonic_status_base::{StatusBackend, MessageKind};
        use tectonic_io_base::IoStack;
        use std::sync::{Arc, Mutex, Condvar};
        
        let (cmd_tx, cmd_rx) = std::sync::mpsc::channel();
        let (event_tx, event_rx) = std::sync::mpsc::channel();
        let tex_path = self.tex_path.clone();

        // Control primitive for stepping
        let control = Arc::new((Mutex::new(false), Condvar::new()));
        let control_clone = control.clone();
        
        // Tracked hashes
        let hashes = Arc::new(Mutex::new(std::collections::HashMap::new()));
        let hashes_clone = hashes.clone();

        std::thread::spawn(move || {
            // Custom StatusBackend that forwards messages to DAP events
            struct EventStatusBackend {
                tx: std::sync::mpsc::Sender<EngineEvent>,
                shadow_vars: std::collections::HashMap<String, String>,
            }
            
            impl StatusBackend for EventStatusBackend {
                fn report(&mut self, kind: MessageKind, args: std::fmt::Arguments<'_>, err: Option<&mut dyn std::error::Error>) {
                    let prefix = match kind {
                        MessageKind::Note => "📝",
                        MessageKind::Warning => "⚠️",
                        MessageKind::Error => "❌",
                    };
                    let msg = if let Some(e) = err {
                        format!("{} {}: {}\n", prefix, args, e)
                    } else {
                        format!("{} {}\n", prefix, args)
                    };

                    // Shadow state parsing: Look for patterns like "{changing \count0=10}"
                    // Note: Tectonic trace output usually goes through the status backend
                    let msg_str = format!("{}", args);
                    if msg_str.starts_with("{changing ") && msg_str.ends_with('}') {
                        let inner = &msg_str[10..msg_str.len()-1];
                        if let Some((var, val)) = inner.split_once('=') {
                            self.shadow_vars.insert(var.to_string(), val.to_string());
                            let _ = self.tx.send(EngineEvent::VariablesUpdated(self.shadow_vars.clone()));
                        }
                    }

                    let _ = self.tx.send(EngineEvent::Output(msg));
                }
                
                fn report_error(&mut self, err: &dyn std::error::Error) {
                    let _ = self.tx.send(EngineEvent::Output(format!("❌ Error: {}\n", err)));
                }
                
                fn dump_error_logs(&mut self, _output: &[u8]) { }
            }
            
            let mut status = EventStatusBackend { 
                tx: event_tx.clone(),
                shadow_vars: std::collections::HashMap::new(),
            };
            
            // Wait for initial launch command
            if let Ok(cmd) = cmd_rx.recv() {
                if !matches!(cmd, EngineCommand::Continue | EngineCommand::Step) {
                    return;
                }
                
                let _ = event_tx.send(EngineEvent::Output("🚀 Starting Tectonic Stepping Engine...\n".to_string()));
                
                let config = match PersistentConfig::open(false) {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = event_tx.send(EngineEvent::Output(format!("❌ Config error: {:?}\n", e)));
                        let _ = event_tx.send(EngineEvent::Terminated);
                        return;
                    }
                };
                
                let bundle = match config.default_bundle(false, &mut status) {
                    Ok(b) => b,
                    Err(e) => {
                        let _ = event_tx.send(EngineEvent::Output(format!("❌ Bundle error: {:?}\n", e)));
                        let _ = event_tx.send(EngineEvent::Terminated);
                        return;
                    }
                };
                
                let output_dir = tex_path.parent().unwrap_or(std::path::Path::new("."));
                
                // Set initial control state to allow first pass
                {
                    let (lock, cvar) = &*control_clone;
                    let mut started = lock.lock().unwrap();
                    *started = true;
                    cvar.notify_all();
                }

                // Create the Stepping Provider
                let base_io = bundle.make_local_io(output_dir, &mut status).unwrap();
                let tex_name = tex_path.file_name().unwrap().to_str().unwrap().to_string();
                let stepping_io = stepping_io::SteppingIoProvider::new(
                    base_io, 
                    event_tx.clone(), 
                    control_clone.clone(),
                    hashes_clone.clone(),
                    Some(tex_name.clone()),
                );
                
                let mut builder = ProcessingSessionBuilder::new_with_security(tectonic::SecuritySettings::new(tectonic::SecurityStance::DisableInsecures));
                builder
                    .primary_input_path(&tex_path)
                    .tex_input_name(&tex_name)
                    .output_format(OutputFormat::Pdf)
                    .output_dir(output_dir)
                    .pass(PassSetting::Default)
                    .filesystem_io(stepping_io); // Use our custom I/O
                
                let mut session = match builder.create(&mut status) {
                    Ok(s) => s,
                    Err(e) => {
                        let _ = event_tx.send(EngineEvent::Output(format!("❌ Session error: {:?}\n", e)));
                        let _ = event_tx.send(EngineEvent::Terminated);
                        return;
                    }
                };
                
                // Thread to handle DAP commands and unblock I/O
                let control_for_cmds = control_clone.clone();
                let event_tx_for_cmds = event_tx.clone();
                std::thread::spawn(move || {
                    while let Ok(cmd) = cmd_rx.recv() {
                        match cmd {
                            EngineCommand::Continue | EngineCommand::Step => {
                                let (lock, cvar) = &*control_for_cmds;
                                let mut started = lock.lock().unwrap();
                                *started = true;
                                cvar.notify_all();
                            }
                            EngineCommand::Terminate => break,
                            _ => {}
                        }
                    }
                    let _ = event_tx_for_cmds.send(EngineEvent::Terminated);
                });

                // Run the session - it will block in stepping_io when files are opened
                match session.run(&mut status) {
                    Ok(_) => {
                        let _ = event_tx.send(EngineEvent::Output("✅ Finished!\n".to_string()));
                        
                        // Emit lockfile info
                        let lock_data = hashes_clone.lock().unwrap();
                        let mut lockfile = ferrotex_build::Lockfile::new();
                        for (path, hash) in lock_data.iter() {
                            lockfile.entries.insert(path.clone(), hash.clone());
                        }
                        
                        // Save to ferrotex.lock in the same directory as the tex file
                        let lock_path = tex_path.with_extension("lock");
                        if let Err(e) = lockfile.save(&lock_path) {
                           let _ = event_tx.send(EngineEvent::Output(format!("⚠️ Failed to save lockfile: {:?}\n", e)));
                        } else {
                           let _ = event_tx.send(EngineEvent::Output(format!("🔐 Saved lockfile to: {}\n", lock_path.display())));
                        }
                    }
                    Err(e) => {
                        let _ = event_tx.send(EngineEvent::Output(format!("❌ Failed: {:?}\n", e)));
                    }
                }
            }
        });

        (cmd_tx, event_rx)
    }
}
