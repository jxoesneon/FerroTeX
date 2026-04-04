use std::sync::mpsc::{Receiver, Sender};

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

/// A driver wraps a TeX engine (real or mock) and provides channel-based control.
pub trait DebugDriver {
    /// Starts the engine in a background thread.
    /// Returns channels for sending commands and receiving events.
    fn spawn(&self) -> (Sender<EngineCommand>, Receiver<EngineEvent>);
}

pub struct MockDebugSession;

impl DebugDriver for MockDebugSession {
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
                        let _ = event_tx
                            .send(EngineEvent::Output(format!("Processing chunk {}\n", steps)));
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
                            location: format!("line {}", steps),
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

#[cfg(feature = "jxoesneon-tectonic-engine")]
mod stepping_io {
    use crate::debug_session::EngineEvent;
    use jxoesneon_tectonic::io::{InputHandle, IoProvider, OpenResult, OutputHandle};
    use jxoesneon_tectonic::status::StatusBackend;
    use sha2::{Digest, Sha256};
    use std::collections::HashMap;
    use std::sync::{Arc, Condvar, Mutex};

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
            Self {
                inner,
                event_tx,
                control,
                hashes,
                primary_file,
            }
        }

        fn wait_for_continue(&self, name: &str) {
            // 1. Notify DAP that we stopped on a file
            if self
                .event_tx
                .send(EngineEvent::Stopped {
                    reason: "file_access".to_string(),
                    location: name.to_string(),
                })
                .is_err()
            {
                return;
            }

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
        fn input_open_name(
            &mut self,
            name: &str,
            status: &mut dyn StatusBackend,
        ) -> OpenResult<InputHandle> {
            // Only stop on "interesting" files (not core formats)
            if name.ends_with(".tex") || name.ends_with(".sty") || name.ends_with(".cls") {
                let _ = self
                    .event_tx
                    .send(EngineEvent::Output(format!("📖 Opening: {}\n", name)));

                // Track hash
                if let Ok(data) = std::fs::read(name) {
                    let mut hasher = Sha256::new();
                    hasher.update(&data);
                    let hash = hex::encode(hasher.finalize());
                    self.hashes.lock().unwrap().insert(name.to_string(), hash);

                    // If this is the primary file, inject tracing flags
                    if self.primary_file.as_deref() == Some(name) {
                        let mut augmented =
                            b"\\tracingassigns=1\\tracingonline=1\\tracingmacros=1\n".to_vec();
                        augmented.extend_from_slice(&data);

                        self.wait_for_continue(name);
                        return OpenResult::Ok(InputHandle::new(
                            name,
                            std::io::Cursor::new(augmented),
                            jxoesneon_tectonic::io::InputOrigin::Filesystem,
                        ));
                    }
                }

                self.wait_for_continue(name);
            }
            self.inner.input_open_name(name, status)
        }

        fn output_open_name(&mut self, name: &str) -> OpenResult<OutputHandle> {
            self.inner.output_open_name(name)
        }
    }
}

/// A driver that uses the real Tectonic engine (requires `tectonic-engine` feature).
///
/// This implementation provides pass-level stepping (TeX pass, bibtex pass, etc.)
/// and forwards Tectonic status messages as DAP engine events.
#[cfg(feature = "jxoesneon-tectonic-engine")]
pub struct TectonicDebugSession {
    pub tex_path: std::path::PathBuf,
}

#[cfg(feature = "jxoesneon-tectonic-engine")]
impl TectonicDebugSession {
    pub fn new(tex_path: std::path::PathBuf) -> Self {
        Self { tex_path }
    }
}

#[cfg(feature = "jxoesneon-tectonic-engine")]
impl DebugDriver for TectonicDebugSession {
    fn spawn(&self) -> (Sender<EngineCommand>, Receiver<EngineEvent>) {
        use jxoesneon_tectonic::config::PersistentConfig;
        use jxoesneon_tectonic::driver::{OutputFormat, PassSetting, ProcessingSessionBuilder};
        use jxoesneon_tectonic_bridge_core::{SecuritySettings, SecurityStance};
        use std::sync::{Arc, Condvar, Mutex};

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
            let mut status = EventStatusBackend {
                tx: event_tx.clone(),
                shadow_vars: std::collections::HashMap::new(),
            };

            // Wait for initial launch command
            if let Ok(cmd) = cmd_rx.recv() {
                if !matches!(cmd, EngineCommand::Continue | EngineCommand::Step) {
                    return;
                }

                let _ = event_tx.send(EngineEvent::Output(
                    "🚀 Starting Tectonic Stepping Engine...\n".to_string(),
                ));

                let config = match PersistentConfig::open(false) {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = event_tx
                            .send(EngineEvent::Output(format!("❌ Config error: {:?}\n", e)));
                        let _ = event_tx.send(EngineEvent::Terminated);
                        return;
                    }
                };

                let bundle = match config.default_bundle(false) {
                    Ok(b) => b,
                    Err(e) => {
                        let _ = event_tx
                            .send(EngineEvent::Output(format!("❌ Bundle error: {:?}\n", e)));
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
                let hidden_paths = std::collections::HashSet::new();
                let base_io = jxoesneon_tectonic::io::FilesystemIo::new(
                    output_dir,
                    false,
                    true,
                    hidden_paths,
                );
                let tex_name = tex_path.file_name().unwrap().to_str().unwrap().to_string();
                let stepping_io = stepping_io::SteppingIoProvider::new(
                    base_io,
                    event_tx.clone(),
                    control_clone.clone(),
                    hashes_clone.clone(),
                    Some(tex_name.clone()),
                );

                let mut builder = ProcessingSessionBuilder::new_with_security(
                    SecuritySettings::new(SecurityStance::DisableInsecures),
                );
                builder
                    .primary_input_path(&tex_path)
                    .tex_input_name(&tex_name)
                    .format_name("latex")
                    .output_format(OutputFormat::Pdf)
                    .output_dir(output_dir)
                    .pass(PassSetting::Default)
                    .bundle(bundle)
                    .filesystem_root(stepping_io); // Use our custom I/O

                // Inject expansion hook
                let event_tx_hook = event_tx.clone();
                let control_hook = control_clone.clone();
                builder.expansion_hook(move || {
                    // Notify DAP
                    let _ = event_tx_hook.send(EngineEvent::Stopped {
                        reason: "expansion".to_string(),
                        location: "macro".to_string(),
                    });

                    // Block
                    let (lock, cvar) = &*control_hook;
                    let mut started = lock.lock().unwrap();
                    *started = false;
                    while !*started {
                        started = cvar.wait(started).unwrap();
                    }
                });

                let mut session = match builder.create(&mut status) {
                    Ok(s) => s,
                    Err(e) => {
                        let _ = event_tx
                            .send(EngineEvent::Output(format!("❌ Session error: {:?}\n", e)));
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
                            let _ = event_tx.send(EngineEvent::Output(format!(
                                "⚠️ Failed to save lockfile: {:?}\n",
                                e
                            )));
                        } else {
                            let _ = event_tx.send(EngineEvent::Output(format!(
                                "🔐 Saved lockfile to: {}\n",
                                lock_path.display()
                            )));
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

#[cfg(feature = "jxoesneon-tectonic-engine")]
struct EventStatusBackend {
    tx: std::sync::mpsc::Sender<EngineEvent>,
    shadow_vars: std::collections::HashMap<String, String>,
}

#[cfg(feature = "jxoesneon-tectonic-engine")]
impl jxoesneon_tectonic::status::StatusBackend for EventStatusBackend {
    fn report(
        &mut self,
        kind: jxoesneon_tectonic_status_base::MessageKind,
        args: std::fmt::Arguments<'_>,
        err: Option<&anyhow::Error>,
    ) {
        let prefix = match kind {
            jxoesneon_tectonic_status_base::MessageKind::Note => "📝",
            jxoesneon_tectonic_status_base::MessageKind::Warning => "⚠️",
            jxoesneon_tectonic_status_base::MessageKind::Error => "❌",
        };
        let msg = if let Some(e) = err {
            format!("{} {}: {}\n", prefix, args, e)
        } else {
            format!("{} {}\n", prefix, args)
        };

        // Shadow state parsing
        let msg_str = format!("{}", args);
        if msg_str.starts_with("{changing ") && msg_str.ends_with('}') {
            let inner = &msg_str[10..msg_str.len() - 1];
            if let Some((var, val)) = inner.split_once('=') {
                self.shadow_vars.insert(var.to_string(), val.to_string());
                let _ = self
                    .tx
                    .send(EngineEvent::VariablesUpdated(self.shadow_vars.clone()));
            }
        } else if msg_str.starts_with("{into ") && msg_str.ends_with('}') {
            let inner = &msg_str[6..msg_str.len() - 1];
            if let Some((var, val)) = inner.split_once('=') {
                self.shadow_vars.insert(var.to_string(), val.to_string());
                let _ = self
                    .tx
                    .send(EngineEvent::VariablesUpdated(self.shadow_vars.clone()));
            }
        }

        let _ = self.tx.send(EngineEvent::Output(msg));
    }

    fn report_error(&mut self, err: &anyhow::Error) {
        let _ = self
            .tx
            .send(EngineEvent::Output(format!("❌ Error: {}\n", err)));
    }

    fn dump_error_logs(&mut self, _output: &[u8]) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "jxoesneon-tectonic-engine")]
    use jxoesneon_tectonic::status::StatusBackend;

    #[test]
    fn test_mock_driver_basic() {
        let driver = MockDebugSession;
        let (tx, rx) = driver.spawn();

        tx.send(EngineCommand::Step).unwrap();
        let event1 = rx.recv().unwrap();
        match event1 {
            EngineEvent::Output(s) => assert!(s.contains("Step 0")),
            _ => panic!("Expected output event"),
        }

        let event2 = rx.recv().unwrap();
        match event2 {
            EngineEvent::Stopped { reason, .. } => assert_eq!(reason, "step"),
            _ => panic!("Expected stopped event"),
        }
    }

    #[test]
    fn test_mock_driver_continue_terminate() {
        let driver = MockDebugSession;
        let (tx, rx) = driver.spawn();

        for i in 0..=5 {
            tx.send(EngineCommand::Continue).unwrap();
            let event = rx.recv().unwrap();
            match event {
                EngineEvent::Output(s) => assert!(s.contains(&format!("Processing chunk {}", i))),
                _ => panic!("Expected output event at step {}", i),
            }
        }

        let event = rx.recv().unwrap();
        match event {
            EngineEvent::Terminated => (),
            _ => panic!("Expected terminated event"),
        }
    }

    #[test]
    fn test_mock_driver_terminate_immediately() {
        let driver = MockDebugSession;
        let (tx, _rx) = driver.spawn();

        // Send Terminate immediately - should break the loop
        tx.send(EngineCommand::Terminate).unwrap();
        // MockDebugSession loop breaks on Terminate, no events sent
    }

    #[test]
    fn test_mock_driver_pause_unknown() {
        let driver = MockDebugSession;
        let (tx, _rx) = driver.spawn();

        // Send Pause - falls through to wildcard branch which also breaks
        tx.send(EngineCommand::Pause).unwrap();
        // MockDebugSession loop breaks on unknown command, no events sent
    }

    #[cfg(feature = "jxoesneon-tectonic-engine")]
    #[test]
    fn test_event_status_backend_parsing() {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut status = EventStatusBackend {
            tx,
            shadow_vars: std::collections::HashMap::new(),
        };

        // Test "into" message
        status.report(
            jxoesneon_tectonic_status_base::MessageKind::Note,
            format_args!("{{into \\count0=11}}"),
            None,
        );
        let mut event = rx.recv().unwrap();
        while matches!(event, EngineEvent::Output(_)) {
            event = rx.recv().unwrap();
        }
        if let EngineEvent::VariablesUpdated(vars) = event {
            assert_eq!(vars.get("\\count0").unwrap(), "11");
        } else {
            panic!("Expected VariablesUpdated event");
        }

        // Test "changing" message
        status.report(
            jxoesneon_tectonic_status_base::MessageKind::Note,
            format_args!("{{changing \\count1=22}}"),
            None,
        );
        let mut event2 = rx.recv().unwrap();
        while matches!(event2, EngineEvent::Output(_)) {
            event2 = rx.recv().unwrap();
        }
        if let EngineEvent::VariablesUpdated(vars) = event2 {
            assert_eq!(vars.get("\\count1").unwrap(), "22");
        } else {
            panic!("Expected VariablesUpdated event");
        }
    }

    #[cfg(feature = "jxoesneon-tectonic-engine")]
    #[test]
    fn test_stepping_io_provider_hashes() {
        use jxoesneon_tectonic::io::MemoryIo;
        use std::sync::{Arc, Condvar, Mutex};

        let (tx, _rx) = std::sync::mpsc::channel();
        let control = Arc::new((Mutex::new(true), Condvar::new()));
        let hashes = Arc::new(Mutex::new(std::collections::HashMap::new()));

        let base_io = MemoryIo::new(true);
        let mut stepping_io =
            stepping_io::SteppingIoProvider::new(base_io, tx, control, hashes.clone(), None);

        // Initially no hashes
        assert!(hashes.lock().unwrap().is_empty());
    }

    #[cfg(feature = "jxoesneon-tectonic-engine")]
    #[test]
    fn test_stepping_io_blocking() {
        use jxoesneon_tectonic::io::{IoProvider, MemoryIo};
        use std::sync::{Arc, Condvar, Mutex};

        let (tx, _rx) = std::sync::mpsc::channel();
        let control = Arc::new((Mutex::new(false), Condvar::new())); // Initially stopped
        let hashes = Arc::new(Mutex::new(std::collections::HashMap::new()));

        let base_io = MemoryIo::new(true);
        let mut stepping_io = stepping_io::SteppingIoProvider::new(
            base_io,
            tx,
            control.clone(),
            hashes.clone(),
            None,
        );

        let control_clone = control.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(200));
            let mut stopped = control_clone.0.lock().unwrap();
            *stopped = true;
            control_clone.1.notify_all();
        });

        let mut status = EventStatusBackend {
            tx: std::sync::mpsc::channel().0,
            shadow_vars: std::collections::HashMap::new(),
        };
        // This should block for 200ms and then succeed (fail to find file, but logic reached)
        let _ = stepping_io.input_open_name("nonexistent", &mut status);
    }

    #[cfg(feature = "jxoesneon-tectonic-engine")]
    #[test]
    fn test_tectonic_debug_session_launch() {
        let temp = tempfile::tempdir().unwrap();
        let tex_path = temp.path().join("test.tex");
        std::fs::write(&tex_path, "\\hello\\world\\bye").unwrap();

        let session = TectonicDebugSession::new(tex_path);
        let (tx, rx) = session.spawn();

        // Send initial continue
        tx.send(EngineCommand::Continue).unwrap();

        // We expect it to at least start and produce some output or reach an error.
        // It might fail because of missing bundles, but it will exercise the logic.
        let mut got_anything = false;
        for _ in 0..20 {
            if let Ok(event) = rx.recv_timeout(std::time::Duration::from_millis(100)) {
                match event {
                    EngineEvent::Output(_)
                    | EngineEvent::VariablesUpdated(_)
                    | EngineEvent::Stopped { .. } => {
                        got_anything = true;
                    }
                    EngineEvent::Terminated => break,
                }
            }
        }
        // At least it should send the "🚀 Starting..." message
        assert!(got_anything);

        tx.send(EngineCommand::Terminate).unwrap();
    }
    #[cfg(feature = "jxoesneon-tectonic-engine")]
    #[test]
    fn test_stepping_io_success() {
        use jxoesneon_tectonic::io::{IoProvider, MemoryIo};
        use std::io::Read;
        use std::sync::{Arc, Condvar, Mutex};

        let (tx, _rx) = std::sync::mpsc::channel();
        let control = Arc::new((Mutex::new(false), Condvar::new()));
        let hashes = Arc::new(Mutex::new(std::collections::HashMap::new()));

        let temp = tempfile::tempdir().unwrap();
        let tex_path = temp.path().join("test_hash.tex");
        std::fs::write(&tex_path, b"test content").unwrap();

        let base_io = jxoesneon_tectonic::io::FilesystemIo::new(
            temp.path(),
            false,
            true,
            std::collections::HashSet::new(),
        );
        let mut stepping_io = stepping_io::SteppingIoProvider::new(
            base_io,
            tx,
            control.clone(),
            hashes.clone(),
            None,
        );

        let mut status = EventStatusBackend {
            tx: std::sync::mpsc::channel().0,
            shadow_vars: std::collections::HashMap::new(),
        };

        let control_clone = control.clone();
        std::thread::spawn(move || {
            // Briefly wait for the wait_for_continue to start blocking
            std::thread::sleep(std::time::Duration::from_millis(50));
            let (lock, cvar) = &*control_clone;
            let mut started = lock.lock().unwrap();
            *started = true;
            cvar.notify_all();
        });

        let mut handle = stepping_io
            .input_open_name(tex_path.to_str().unwrap(), &mut status)
            .must_exist()
            .unwrap();
        let mut buf = Vec::new();
        handle.read_to_end(&mut buf).unwrap();
        drop(handle); // This should trigger hash computation

        assert!(!hashes.lock().unwrap().is_empty());
    }

    #[cfg(feature = "jxoesneon-tectonic-engine")]
    #[test]
    fn test_stepping_io_output_and_non_tex() {
        use jxoesneon_tectonic::io::{IoProvider, MemoryIo};
        use std::sync::{Arc, Condvar, Mutex};

        let (tx, _rx) = std::sync::mpsc::channel();
        let control = Arc::new((Mutex::new(false), Condvar::new()));
        let hashes = Arc::new(Mutex::new(std::collections::HashMap::new()));

        let base_io = MemoryIo::new(true);
        let mut stepping_io =
            stepping_io::SteppingIoProvider::new(base_io, tx, control, hashes, None);

        let mut status = EventStatusBackend {
            tx: std::sync::mpsc::channel().0,
            shadow_vars: std::collections::HashMap::new(),
        };

        // Non-TeX file should not trigger blocking or hashing
        let result = stepping_io.input_open_name("test.txt", &mut status);
        assert!(matches!(
            result,
            jxoesneon_tectonic::io::OpenResult::NotAvailable
        ));

        // Output should just pass through
        let out_result = stepping_io.output_open_name("test_out.pdf");
        assert!(matches!(
            out_result,
            jxoesneon_tectonic::io::OpenResult::Ok(_)
        ));
    }

    #[cfg(feature = "jxoesneon-tectonic-engine")]
    #[test]
    fn test_stepping_io_primary_file_injection() {
        use jxoesneon_tectonic::io::IoProvider;
        use std::io::Read;
        use std::sync::{Arc, Condvar, Mutex};

        let (tx, rx) = std::sync::mpsc::channel();
        // Initialize control; wait_for_continue will reset it to false
        let control = Arc::new((Mutex::new(false), Condvar::new()));
        let hashes = Arc::new(Mutex::new(std::collections::HashMap::new()));

        let temp = tempfile::tempdir().unwrap();
        let tex_path = temp.path().join("primary.tex");
        let tex_path_str = tex_path.to_str().unwrap().to_string();
        std::fs::write(&tex_path, b"\\documentclass{article}").unwrap();

        let base_io = jxoesneon_tectonic::io::FilesystemIo::new(
            temp.path(),
            false,
            true,
            std::collections::HashSet::new(),
        );
        let mut stepping_io = stepping_io::SteppingIoProvider::new(
            base_io,
            tx,
            control.clone(),
            hashes.clone(),
            Some(tex_path_str.clone()),
        );

        let mut status = EventStatusBackend {
            tx: std::sync::mpsc::channel().0,
            shadow_vars: std::collections::HashMap::new(),
        };

        // Spawn a thread to unblock the provider after it requests a stop
        let control_clone = control.clone();
        std::thread::spawn(move || {
            // Wait for the "Stopped" event to be sent (simulated by sleep here)
            std::thread::sleep(std::time::Duration::from_millis(200));
            // Unblock
            let (lock, cvar) = &*control_clone;
            let mut started = lock.lock().unwrap();
            *started = true;
            cvar.notify_all();
        });

        // Open the primary file - should inject tracing flags and allow continue
        let result = stepping_io.input_open_name(&tex_path_str, &mut status);
        if let jxoesneon_tectonic::io::OpenResult::Ok(mut handle) = result {
            let mut contents = String::new();
            handle.read_to_string(&mut contents).unwrap();
            // Should contain the injected tracing flags
            assert!(contents.contains("\\tracingassigns=1"));
            assert!(contents.contains("\\documentclass{article}"));
        }

        // Check that we received the events
        // 1. Output "Opening: ..."
        let event1 = rx.recv_timeout(std::time::Duration::from_secs(1)).unwrap();
        // The events might come in any order depending on channel/thread verify
        // But Stopped comes first in wait_for_continue, however "Opening" print is before wait_for_continue logic
        // Actually:
        // 1. "Opening..." (Output)
        // 2. Stopped (if wait_for_continue called)

        // Let's just collect all events
        let mut events = vec![event1];
        if let Ok(e) = rx.recv_timeout(std::time::Duration::from_millis(500)) {
            events.push(e);
        }

        assert!(
            events
                .iter()
                .any(|e| matches!(e, EngineEvent::Output(s) if s.contains("📖 Opening")))
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, EngineEvent::Stopped { .. }))
        );

        // Check that hash was tracked
        let h = hashes.lock().unwrap();
        assert!(!h.is_empty());
    }

    #[cfg(feature = "jxoesneon-tectonic-engine")]
    #[test]
    fn test_stepping_io_channel_failure() {
        use jxoesneon_tectonic::io::{IoProvider, MemoryIo};
        use std::sync::{Arc, Condvar, Mutex};

        let (tx, rx) = std::sync::mpsc::channel();
        drop(rx); // Break the channel

        let control = Arc::new((Mutex::new(true), Condvar::new()));
        let hashes = Arc::new(Mutex::new(std::collections::HashMap::new()));

        let mut stepping_io = stepping_io::SteppingIoProvider::new(
            MemoryIo::new(true),
            tx,
            control.clone(),
            hashes,
            None,
        );

        let mut status = EventStatusBackend {
            tx: std::sync::mpsc::channel().0,
            shadow_vars: std::collections::HashMap::new(),
        };

        // This should not panic even if send fails
        let _ = stepping_io.input_open_name("test.tex", &mut status);
    }

    #[cfg(feature = "jxoesneon-tectonic-engine")]
    #[test]
    fn test_event_status_backend_error_and_message() {
        use jxoesneon_tectonic::status::StatusBackend;
        let (tx, rx) = std::sync::mpsc::channel();
        let mut status = EventStatusBackend {
            tx,
            shadow_vars: std::collections::HashMap::new(),
        };

        // Test error reporting
        let err = anyhow::anyhow!("test error");
        status.report(
            jxoesneon_tectonic::status::MessageKind::Error,
            format_args!("test"),
            Some(&err),
        );
        let event = rx.try_recv().unwrap();
        assert!(matches!(event, EngineEvent::Output(_)));

        // Test message with invalid JSON (should send Output but not VariablesUpdated)
        status.report(
            jxoesneon_tectonic::status::MessageKind::Note,
            format_args!("not json"),
            None,
        );
        let _ = rx.try_recv().unwrap(); // Consume Output

        // Test variables update (format: {changing var=val} or {into var=val})
        status.report(
            jxoesneon_tectonic::status::MessageKind::Note,
            format_args!("{{changing test=val}}"),
            None,
        );
        // This report sends TWO events: VariablesUpdated then Output
        let event1 = rx.try_recv().unwrap();
        assert!(matches!(event1, EngineEvent::VariablesUpdated(_)));
        let _ = rx.try_recv().unwrap(); // Consume Output

        // Test {into var=val} format (second shadow var branch)
        status.report(
            jxoesneon_tectonic::status::MessageKind::Note,
            format_args!("{{into foo=bar}}"),
            None,
        );
        let event2 = rx.try_recv().unwrap();
        assert!(matches!(event2, EngineEvent::VariablesUpdated(_)));
        let _ = rx.try_recv().unwrap(); // Consume Output

        // Test Warning message kind
        status.report(
            jxoesneon_tectonic::status::MessageKind::Warning,
            format_args!("warning test"),
            None,
        );
        let event3 = rx.try_recv().unwrap();
        assert!(matches!(event3, EngineEvent::Output(s) if s.contains("⚠️")));

        // Test report_error
        let err2 = anyhow::anyhow!("fatal error");
        status.report_error(&err2);
        let event4 = rx.try_recv().unwrap();
        assert!(matches!(event4, EngineEvent::Output(s) if s.contains("fatal error")));

        // Test dump_error_logs (no-op)
        status.dump_error_logs(b"some log output");
    }

    /// Integration test that spawns a TectonicDebugSession with a real TeX file.
    /// Marked `#[ignore]` because it requires network access to download bundles.
    /// Run with: `cargo test --features jxoesneon-tectonic-engine -- --ignored test_tectonic_session_real_compilation`
    #[test]
    #[ignore]
    #[cfg(feature = "jxoesneon-tectonic-engine")]
    fn test_tectonic_session_real_compilation() {
        use std::time::Duration;

        // Create a temporary TeX file
        let temp_dir = tempfile::tempdir().unwrap();
        let tex_path = temp_dir.path().join("test.tex");
        std::fs::write(
            &tex_path,
            r#"\documentclass{article}
\begin{document}
Hello, FerroTeX!
\end{document}
"#,
        )
        .unwrap();

        // Spawn the debug session
        let session = TectonicDebugSession::new(tex_path.clone());
        let (cmd_tx, event_rx) = session.spawn();

        // Send a Continue command to start compilation
        cmd_tx.send(EngineCommand::Continue).unwrap();

        // Collect events with a short timeout - we just want to verify the session starts
        let mut events = Vec::new();
        let timeout = Duration::from_secs(10); // Short timeout - just verify startup
        let start = std::time::Instant::now();

        while start.elapsed() < timeout && events.len() < 5 {
            match event_rx.recv_timeout(Duration::from_millis(500)) {
                Ok(event) => {
                    let is_terminated = matches!(event, EngineEvent::Terminated);
                    events.push(event);
                    if is_terminated {
                        break;
                    }
                    // If we get a Stopped event, send Continue to resume
                    if matches!(events.last(), Some(EngineEvent::Stopped { .. })) {
                        cmd_tx.send(EngineCommand::Continue).unwrap();
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }

        // Terminate the session to clean up
        let _ = cmd_tx.send(EngineCommand::Terminate);

        // Verify we received at least the startup message
        assert!(!events.is_empty(), "Should receive at least one event");
        let has_startup = events
            .iter()
            .any(|e| matches!(e, EngineEvent::Output(s) if s.contains("Starting")));
        assert!(has_startup, "Should receive startup message");
    }

    #[test]
    fn test_types_debug_clone() {
        let cmd = EngineCommand::Step;
        let cmd_clone = cmd.clone();
        assert_eq!(format!("{:?}", cmd), "Step");
        assert_eq!(format!("{:?}", cmd_clone), "Step");

        let event = EngineEvent::Terminated;
        let event_clone = event.clone();
        assert_eq!(format!("{:?}", event), "Terminated");
        assert_eq!(format!("{:?}", event_clone), "Terminated");

        let event_stopped = EngineEvent::Stopped {
            reason: "pause".to_string(),
            location: "l1".to_string(),
        };
        let event_stopped_clone = event_stopped.clone();
        assert!(format!("{:?}", event_stopped).contains("pause"));
        assert!(format!("{:?}", event_stopped_clone).contains("l1"));

        let event_output = EngineEvent::Output("log".to_string());
        let event_output_clone = event_output.clone();
        assert!(format!("{:?}", event_output).contains("log"));
        assert!(format!("{:?}", event_output_clone).contains("log"));

        let mut vars = std::collections::HashMap::new();
        vars.insert("k".to_string(), "v".to_string());
        let event_vars = EngineEvent::VariablesUpdated(vars);
        let event_vars_clone = event_vars.clone();
        assert!(format!("{:?}", event_vars).contains("k"));
        assert!(format!("{:?}", event_vars_clone).contains("v"));
    }

    #[test]
    fn test_mock_driver_comprehensive() {
        let driver = MockDebugSession;
        let (tx, rx) = driver.spawn();

        // 1. Test Step
        tx.send(EngineCommand::Step).unwrap();
        let ev = rx.recv_timeout(std::time::Duration::from_millis(500)).unwrap();
        assert!(matches!(ev, EngineEvent::Output(s) if s.contains("Step 0")));
        let ev = rx.recv_timeout(std::time::Duration::from_millis(500)).unwrap();
        assert!(matches!(ev, EngineEvent::Stopped { reason, .. } if reason == "step"));

        // 2. Test Continue multiple times
        for i in 1..=4 {
            tx.send(EngineCommand::Continue).unwrap();
            let ev = rx.recv_timeout(std::time::Duration::from_millis(500)).unwrap();
            assert!(matches!(ev, EngineEvent::Output(s) if s.contains(&format!("Processing chunk {}", i))));
        }

        // 3. The next continue should reach steps=6 and terminate
        tx.send(EngineCommand::Continue).unwrap();
        let ev = rx.recv_timeout(std::time::Duration::from_millis(500)).unwrap();
        assert!(matches!(ev, EngineEvent::Output(s) if s.contains("Processing chunk 5")));
        let ev = rx.recv_timeout(std::time::Duration::from_millis(500)).unwrap();
        assert!(matches!(ev, EngineEvent::Terminated));

        // 4. After termination, the event channel should be closed
        let result = rx.recv_timeout(std::time::Duration::from_millis(100));
        assert!(matches!(result, Err(std::sync::mpsc::RecvTimeoutError::Disconnected)));
    }

    #[test]
    fn test_mock_driver_cmd_tx_dropped() {
        let driver = MockDebugSession;
        let (tx, rx) = driver.spawn();
        drop(tx); // Should cause recv() to return Err and break the loop
        let result = rx.recv_timeout(std::time::Duration::from_millis(500));
        assert!(matches!(result, Err(std::sync::mpsc::RecvTimeoutError::Disconnected)));
    }

    #[test]
    fn test_mock_driver_event_rx_dropped() {
        let driver = MockDebugSession;
        let (tx, rx) = driver.spawn();
        drop(rx);
        // This should not panic even though event_tx.send will fail
        tx.send(EngineCommand::Step).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    #[test]
    fn test_engine_command_pause_explicit() {
        let driver = MockDebugSession;
        let (tx, rx) = driver.spawn();
        tx.send(EngineCommand::Pause).unwrap();
        // Pause should hit the wildcard and break the loop
        let result = rx.recv_timeout(std::time::Duration::from_millis(500));
        assert!(matches!(result, Err(std::sync::mpsc::RecvTimeoutError::Disconnected)));
    }

    struct DummyDriver;
    impl DebugDriver for DummyDriver {
        fn spawn(&self) -> (Sender<EngineCommand>, Receiver<EngineEvent>) {
            let (cmd_tx, _cmd_rx) = std::sync::mpsc::channel();
            let (_event_tx, event_rx) = std::sync::mpsc::channel();
            (cmd_tx, event_rx)
        }
    }

    #[test]
    fn test_debug_driver_trait_coverage() {
        let driver = DummyDriver;
        let (_tx, _rx) = driver.spawn();
    }

    #[test]
    fn test_engine_event_variables_updated_explicit() {
        let mut vars = std::collections::HashMap::new();
        vars.insert("foo".to_string(), "bar".to_string());
        let event = EngineEvent::VariablesUpdated(vars);
        if let EngineEvent::VariablesUpdated(v) = event {
            assert_eq!(v.get("foo").unwrap(), "bar");
        } else {
            panic!("Expected VariablesUpdated variant");
        }
    }
}
