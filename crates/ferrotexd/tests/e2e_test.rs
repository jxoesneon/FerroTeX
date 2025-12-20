use serde_json::json;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::time::{sleep, timeout};
use tower_lsp::lsp_types::Url;

#[tokio::test]
async fn test_lsp_diagnostics_flow() -> anyhow::Result<()> {
    // 1. Setup temp dir
    let temp_dir = tempfile::tempdir()?;
    let temp_path = temp_dir.path().canonicalize()?;

    // 2. Locate the binary (Assumes `cargo build -p ferrotexd` has been run)
    // We assume the test is run from crate root or workspace root.
    // Let's look for the binary in standard cargo locations relative to current dir.

    let mut bin_path = None;
    let candidates = vec![
        "../../target/debug/ferrotexd", // From crate root
        "target/debug/ferrotexd",       // From workspace root
    ];

    for candidate in candidates {
        let path = std::env::current_dir()?.join(candidate);
        if path.exists() {
            bin_path = Some(path);
            break;
        }
    }

    let final_bin_path = bin_path.ok_or_else(|| {
        anyhow::anyhow!("ferrotexd binary not found. Run `cargo build -p ferrotexd` first.")
    })?;

    // 3. Start the server
    let mut child = Command::new(final_bin_path)
        .current_dir(&temp_path) // Watch this dir
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let stdin = child.stdin.as_mut().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);

    // 5. Send Initialize
    let init_msg = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "capabilities": {},
            "rootUri": Url::from_directory_path(&temp_path).unwrap(),
            "processId": std::process::id()
        }
    });
    let init_str = init_msg.to_string();
    stdin
        .write_all(format!("Content-Length: {}\r\n\r\n{}", init_str.len(), init_str).as_bytes())
        .await?;

    // 6. Read Initialize Result
    read_msg(&mut reader).await?; // Content-Length

    // 7. Send Initialized
    let initialized_msg = json!({
        "jsonrpc": "2.0",
        "method": "initialized",
        "params": {}
    });
    let init_notif = initialized_msg.to_string();
    stdin
        .write_all(format!("Content-Length: {}\r\n\r\n{}", init_notif.len(), init_notif).as_bytes())
        .await?;

    // 8. Create a log file with warnings
    let log_file = temp_path.join("test.log");
    tokio::fs::write(&log_file, "LaTeX Warning: Label `foo' multiply defined.\n").await?;

    // 9. Wait for publishDiagnostics
    // It might take a moment for notify to trigger and processing to happen
    let log_uri = Url::from_file_path(&log_file).unwrap();

    let wait_loop = async {
        loop {
            let msg = read_msg(&mut reader).await?;
            // eprintln!("Received: {:?}", msg); // Debug
            if msg.get("method").and_then(|m| m.as_str()) == Some("textDocument/publishDiagnostics")
            {
                let params = &msg["params"];
                if let Some(uri) = params["uri"].as_str() {
                    if uri == log_uri.as_str() {
                        let diags = params["diagnostics"].as_array().unwrap();
                        if !diags.is_empty() {
                            let msg = diags[0]["message"].as_str().unwrap();
                            if msg.contains("Label `foo' multiply defined") {
                                return Ok::<(), anyhow::Error>(());
                            }
                        }
                    } else {
                        // eprintln!("Ignored diagnostics for uri: {}", uri);
                    }
                }
            }
        }
    };

    match timeout(Duration::from_secs(30), wait_loop).await {
        Ok(Ok(())) => {}
        Ok(Err(e)) => anyhow::bail!("Error reading message: {:?}", e),
        Err(_) => anyhow::bail!(
            "Timed out waiting for log diagnostic. Expected URI: {}",
            log_uri
        ),
    }

    // Cleanup
    child.kill().await?;

    Ok(())
}

#[tokio::test]
async fn test_document_symbol_flow() -> anyhow::Result<()> {
    // 1. Setup temp dir
    let temp_dir = tempfile::tempdir()?;
    let temp_path = temp_dir.path().canonicalize()?;

    // 2. Locate binary
    let mut bin_path = None;
    let candidates = vec!["../../target/debug/ferrotexd", "target/debug/ferrotexd"];
    for candidate in candidates {
        let path = std::env::current_dir()?.join(candidate);
        if path.exists() {
            bin_path = Some(path);
            break;
        }
    }
    let final_bin_path = bin_path.ok_or_else(|| anyhow::anyhow!("ferrotexd binary not found"))?;

    // 3. Start server
    let mut child = Command::new(final_bin_path)
        .current_dir(&temp_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let stdin = child.stdin.as_mut().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);

    // 4. Initialize
    let init_msg = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "capabilities": {},
            "rootUri": Url::from_directory_path(&temp_path).unwrap(),
            "processId": std::process::id()
        }
    });
    send_msg(stdin, &init_msg).await?;

    // Read Initialize Result
    read_msg(&mut reader).await?;

    // Initialized
    let initialized_msg = json!({
        "jsonrpc": "2.0",
        "method": "initialized",
        "params": {}
    });
    send_msg(stdin, &initialized_msg).await?;

    // 5. Open a document
    let doc_path = temp_path.join("main.tex");
    let doc_uri = Url::from_file_path(&doc_path).unwrap();
    let doc_text = r"\begin{document} \begin{itemize} \item A \end{itemize} \end{document}";
    let did_open = json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {
                "uri": doc_uri,
                "languageId": "latex",
                "version": 1,
                "text": doc_text
            }
        }
    });
    send_msg(stdin, &did_open).await?;

    // Wait for server to process didOpen
    sleep(Duration::from_millis(100)).await;

    // 6. Request Document Symbols
    let sym_req = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "textDocument/documentSymbol",
        "params": {
            "textDocument": {
                "uri": doc_uri
            }
        }
    });
    send_msg(stdin, &sym_req).await?;

    // 7. Read Response
    // We might get window/logMessage notifications, so we loop until we get the response to ID 2
    let syms = loop {
        let msg = read_msg(&mut reader).await?;
        if let Some(id) = msg.get("id") {
            if id == 2 {
                break msg["result"]
                    .as_array()
                    .expect("result should be an array")
                    .clone();
            }
        }
        // Ignore other messages (notifications)
    };

    // We expect nested structure: Environment(document) -> Environment(itemize)
    assert!(!syms.is_empty(), "Should return symbols");
    let doc_sym = &syms[0];
    assert_eq!(doc_sym["name"], "document");

    let children = doc_sym["children"].as_array().unwrap();
    assert!(!children.is_empty(), "Document env should have children");

    // The first child is likely the {document} group argument.
    // The second child (or later) is the nested environment.
    let itemize_sym = children
        .iter()
        .find(|s| s["name"] == "itemize")
        .expect("Should find nested itemize symbol");

    assert_eq!(itemize_sym["name"], "itemize");

    // Cleanup
    child.kill().await?;
    Ok(())
}

#[tokio::test]
async fn test_syntax_diagnostics_flow() -> anyhow::Result<()> {
    // 1. Setup temp dir
    let temp_dir = tempfile::tempdir()?;
    let temp_path = temp_dir.path().to_owned();

    // 2. Locate binary
    let mut bin_path = None;
    let candidates = vec!["../../target/debug/ferrotexd", "target/debug/ferrotexd"];
    for candidate in candidates {
        let path = std::env::current_dir()?.join(candidate);
        if path.exists() {
            bin_path = Some(path);
            break;
        }
    }
    let final_bin_path = bin_path.ok_or_else(|| anyhow::anyhow!("ferrotexd binary not found"))?;

    // 3. Start server
    let mut child = Command::new(final_bin_path)
        .current_dir(&temp_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let stdin = child.stdin.as_mut().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);

    // 4. Initialize
    let init_msg = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "capabilities": {},
            "rootUri": format!("file://{}", temp_path.display()),
            "processId": std::process::id()
        }
    });
    send_msg(stdin, &init_msg).await?;
    read_msg(&mut reader).await?; // Result

    let initialized_msg = json!({
        "jsonrpc": "2.0",
        "method": "initialized",
        "params": {}
    });
    send_msg(stdin, &initialized_msg).await?;

    // 5. Open a document with syntax error
    let doc_uri = format!("file://{}/broken.tex", temp_path.display());
    // Missing closing brace
    let doc_text = r"{ \cmd";
    let did_open = json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {
                "uri": doc_uri,
                "languageId": "latex",
                "version": 1,
                "text": doc_text
            }
        }
    });
    send_msg(stdin, &did_open).await?;

    // 6. Wait for diagnostics
    let mut found = false;
    for _ in 0..10 {
        let msg = read_msg(&mut reader).await?;
        if msg["method"] == "textDocument/publishDiagnostics" {
            let params = &msg["params"];
            let uri = params["uri"].as_str().unwrap();
            if uri == doc_uri {
                let diags = params["diagnostics"].as_array().unwrap();
                if !diags.is_empty() {
                    let message = diags[0]["message"].as_str().unwrap();
                    if message.contains("Expected '}'") {
                        found = true;
                        break;
                    }
                }
            }
        }
    }

    assert!(found, "Did not receive syntax error diagnostic");

    // Cleanup
    child.kill().await?;
    Ok(())
}

#[tokio::test]
async fn test_document_symbol_section_flow() -> anyhow::Result<()> {
    // 1. Setup temp dir
    let temp_dir = tempfile::tempdir()?;
    let temp_path = temp_dir.path().to_owned();

    // 2. Locate binary
    let mut bin_path = None;
    let candidates = vec!["../../target/debug/ferrotexd", "target/debug/ferrotexd"];
    for candidate in candidates {
        let path = std::env::current_dir()?.join(candidate);
        if path.exists() {
            bin_path = Some(path);
            break;
        }
    }
    let final_bin_path = bin_path.ok_or_else(|| anyhow::anyhow!("ferrotexd binary not found"))?;

    // 3. Start server
    let mut child = Command::new(final_bin_path)
        .current_dir(&temp_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let stdin = child.stdin.as_mut().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);

    // 4. Initialize
    let init_msg = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "capabilities": {},
            "rootUri": format!("file://{}", temp_path.display()),
            "processId": std::process::id()
        }
    });
    send_msg(stdin, &init_msg).await?;
    read_msg(&mut reader).await?;

    let initialized_msg = json!({
        "jsonrpc": "2.0",
        "method": "initialized",
        "params": {}
    });
    send_msg(stdin, &initialized_msg).await?;

    // 5. Open a document with sections
    let doc_uri = format!("file://{}/sections.tex", temp_path.display());
    let doc_text = r"\section{Introduction} \begin{itemize} \item A \end{itemize}";
    let did_open = json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {
                "uri": doc_uri,
                "languageId": "latex",
                "version": 1,
                "text": doc_text
            }
        }
    });
    send_msg(stdin, &did_open).await?;

    // 6. Request Document Symbols
    let sym_req = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "textDocument/documentSymbol",
        "params": {
            "textDocument": {
                "uri": doc_uri
            }
        }
    });
    send_msg(stdin, &sym_req).await?;

    // 7. Read Response
    let syms = loop {
        let msg = read_msg(&mut reader).await?;
        if let Some(id) = msg.get("id") {
            if id == 2 {
                break msg["result"]
                    .as_array()
                    .expect("result should be an array")
                    .clone();
            }
        }
    };

    // Expect: Section, then Environment
    assert_eq!(
        syms.len(),
        2,
        "Should return Section and Environment symbols"
    );

    let section_sym = &syms[0];
    assert_eq!(section_sym["name"], "Introduction");

    let env_sym = &syms[1];
    assert_eq!(env_sym["name"], "itemize");

    // Cleanup
    child.kill().await?;
    Ok(())
}

#[tokio::test]
async fn test_document_link_flow() -> anyhow::Result<()> {
    // 1. Setup temp dir
    let temp_dir = tempfile::tempdir()?;
    let temp_path = temp_dir.path().to_owned();

    // 2. Locate binary
    let mut bin_path = None;
    let candidates = vec!["../../target/debug/ferrotexd", "target/debug/ferrotexd"];
    for candidate in candidates {
        let path = std::env::current_dir()?.join(candidate);
        if path.exists() {
            bin_path = Some(path);
            break;
        }
    }
    let final_bin_path = bin_path.ok_or_else(|| anyhow::anyhow!("ferrotexd binary not found"))?;

    // 3. Start server
    let mut child = Command::new(final_bin_path)
        .current_dir(&temp_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let stdin = child.stdin.as_mut().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);

    // 4. Initialize
    let init_msg = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "capabilities": {},
            "rootUri": format!("file://{}", temp_path.display()),
            "processId": std::process::id()
        }
    });
    send_msg(stdin, &init_msg).await?;
    read_msg(&mut reader).await?;

    let initialized_msg = json!({
        "jsonrpc": "2.0",
        "method": "initialized",
        "params": {}
    });
    send_msg(stdin, &initialized_msg).await?;

    // 5. Open a document with includes
    let doc_uri = format!("file://{}/main.tex", temp_path.display());
    let doc_text = r"\documentclass{article} \input{chapters/intro} \include{chapters/concl}";
    let did_open = json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {
                "uri": doc_uri,
                "languageId": "latex",
                "version": 1,
                "text": doc_text
            }
        }
    });
    send_msg(stdin, &did_open).await?;

    // 6. Request Document Links
    let link_req = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "textDocument/documentLink",
        "params": {
            "textDocument": {
                "uri": doc_uri
            }
        }
    });
    send_msg(stdin, &link_req).await?;

    // 7. Read Response
    let links = loop {
        let msg = read_msg(&mut reader).await?;
        if let Some(id) = msg.get("id") {
            if id == 2 {
                break msg["result"]
                    .as_array()
                    .expect("result should be an array")
                    .clone();
            }
        }
    };

    assert_eq!(links.len(), 2, "Should return 2 links");

    // Check first link (input)
    let link1 = &links[0];
    let target1 = link1["target"].as_str().unwrap();
    assert!(
        target1.ends_with("chapters/intro"),
        "Target should end with chapters/intro"
    );

    // Check second link (include)
    let link2 = &links[1];
    let target2 = link2["target"].as_str().unwrap();
    assert!(
        target2.ends_with("chapters/concl"),
        "Target should end with chapters/concl"
    );

    // Cleanup
    child.kill().await?;
    Ok(())
}

#[tokio::test]
async fn test_cycle_detection_flow() -> anyhow::Result<()> {
    // 1. Setup temp dir
    let temp_dir = tempfile::tempdir()?;
    let temp_path = temp_dir.path().to_owned();

    // 2. Locate binary
    let mut bin_path = None;
    let candidates = vec!["../../target/debug/ferrotexd", "target/debug/ferrotexd"];
    for candidate in candidates {
        let path = std::env::current_dir()?.join(candidate);
        if path.exists() {
            bin_path = Some(path);
            break;
        }
    }
    let final_bin_path = bin_path.ok_or_else(|| anyhow::anyhow!("ferrotexd binary not found"))?;

    // 3. Start server
    let mut child = Command::new(final_bin_path)
        .current_dir(&temp_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let stdin = child.stdin.as_mut().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);

    // 4. Initialize
    let init_msg = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "capabilities": {},
            "rootUri": format!("file://{}", temp_path.display()),
            "processId": std::process::id()
        }
    });
    send_msg(stdin, &init_msg).await?;
    read_msg(&mut reader).await?;

    let initialized_msg = json!({
        "jsonrpc": "2.0",
        "method": "initialized",
        "params": {}
    });
    send_msg(stdin, &initialized_msg).await?;

    // 5. Open A -> B
    let uri_a = format!("file://{}/a.tex", temp_path.display());
    let text_a = r"\input{b.tex}";
    let did_open_a = json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {
                "uri": uri_a,
                "languageId": "latex",
                "version": 1,
                "text": text_a
            }
        }
    });
    send_msg(stdin, &did_open_a).await?;

    // We might get diagnostics for A (empty or syntax errors), consume them if any
    // Wait a bit or consume until idle? simpler to just proceed since we check B specifically.

    // 6. Open B -> A (Cycle!)
    let uri_b = format!("file://{}/b.tex", temp_path.display());
    let text_b = r"\input{a.tex}";
    let did_open_b = json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {
                "uri": uri_b,
                "languageId": "latex",
                "version": 1,
                "text": text_b
            }
        }
    });
    send_msg(stdin, &did_open_b).await?;

    // 7. Wait for Cycle Diagnostic on B
    let mut found = false;
    for _ in 0..10 {
        let msg = read_msg(&mut reader).await?;
        if msg["method"] == "textDocument/publishDiagnostics" {
            let params = &msg["params"];
            let uri = params["uri"].as_str().unwrap();
            if uri == uri_b {
                let diags = params["diagnostics"].as_array().unwrap();
                for d in diags {
                    let message = d["message"].as_str().unwrap();
                    if message.contains("Cycle detected") && message.contains("a.tex") {
                        found = true;
                    }
                }
                if found {
                    break;
                }
            }
        }
    }

    assert!(found, "Did not receive cycle detected diagnostic on b.tex");

    // Cleanup
    child.kill().await?;
    Ok(())
}

#[tokio::test]
async fn test_label_features_flow() -> anyhow::Result<()> {
    // 1. Setup temp dir
    let temp_dir = tempfile::tempdir()?;
    let temp_path = temp_dir.path().to_owned();

    // 2. Locate binary
    let mut bin_path = None;
    let candidates = vec!["../../target/debug/ferrotexd", "target/debug/ferrotexd"];
    for candidate in candidates {
        let path = std::env::current_dir()?.join(candidate);
        if path.exists() {
            bin_path = Some(path);
            break;
        }
    }
    let final_bin_path = bin_path.ok_or_else(|| anyhow::anyhow!("ferrotexd binary not found"))?;

    // 3. Start server
    let mut child = Command::new(final_bin_path)
        .current_dir(&temp_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let stdin = child.stdin.as_mut().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);

    // 4. Initialize
    let init_msg = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "capabilities": {},
            "rootUri": format!("file://{}", temp_path.display()),
            "processId": std::process::id()
        }
    });
    send_msg(stdin, &init_msg).await?;
    let _init_res = read_msg(&mut reader).await?;

    let initialized_msg = json!({
        "jsonrpc": "2.0",
        "method": "initialized",
        "params": {}
    });
    send_msg(stdin, &initialized_msg).await?;

    // 5. Open document
    let doc_uri = format!("file://{}/main.tex", temp_path.display());
    let doc_text = "\\section{Intro}\n\\label{sec:intro}\nSee Section \\ref{sec:intro}.";
    let did_open = json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {
                "uri": doc_uri,
                "languageId": "latex",
                "version": 1,
                "text": doc_text
            }
        }
    });
    send_msg(stdin, &did_open).await?;

    // Wait for diagnostics as signal that file is processed
    let mut indexed = false;
    for _ in 0..20 {
        let msg = timeout(Duration::from_secs(1), read_msg(&mut reader)).await??;
        if msg["method"] == "textDocument/publishDiagnostics" {
            // We got diagnostics, meaning validate_document ran, so indexing ran.
            indexed = true;
            break;
        }
    }
    assert!(indexed, "Timed out waiting for initial indexing");

    // 6. Test Goto Definition
    let def_req = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "textDocument/definition",
        "params": {
            "textDocument": { "uri": doc_uri },
            "position": { "line": 2, "character": 20 }
        }
    });
    send_msg(stdin, &def_req).await?;

    let def_res = loop {
        let msg = read_msg(&mut reader).await?;
        if let Some(id) = msg.get("id") {
            if id == 2 {
                break msg;
            }
        }
    };

    let locs = def_res["result"]
        .as_array()
        .expect("Definition result should be array");
    assert!(!locs.is_empty(), "Should find definition");
    let loc = &locs[0];
    let range = loc["range"].as_object().unwrap();
    let start = range["start"].as_object().unwrap();
    assert_eq!(start["line"], 1, "Definition should be on line 1");

    // 7. Test References
    let ref_req = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "textDocument/references",
        "params": {
            "textDocument": { "uri": doc_uri },
            "position": { "line": 1, "character": 10 },
            "context": { "includeDeclaration": true }
        }
    });
    send_msg(stdin, &ref_req).await?;

    let ref_res = loop {
        let msg = read_msg(&mut reader).await?;
        if let Some(id) = msg.get("id") {
            if id == 3 {
                break msg;
            }
        }
    };
    let refs = ref_res["result"]
        .as_array()
        .expect("References result should be array");
    assert_eq!(refs.len(), 2, "Should find 2 locations");

    // 8. Test Diagnostics: Undefined Reference
    let new_text = format!("{}\n\\ref{{missing}}", doc_text);
    let did_change = json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didChange",
        "params": {
            "textDocument": { "uri": doc_uri, "version": 2 },
            "contentChanges": [ { "text": new_text } ]
        }
    });
    send_msg(stdin, &did_change).await?;

    let wait_loop = async {
        loop {
            let msg = read_msg(&mut reader).await?;
            if msg["method"] == "textDocument/publishDiagnostics" {
                let params = &msg["params"];
                let uri = params["uri"].as_str().unwrap();
                if uri == doc_uri {
                    let diags = params["diagnostics"].as_array().unwrap();
                    for d in diags {
                        if let Some(msg) = d["message"].as_str() {
                            if msg.contains("Undefined reference: 'missing'") {
                                return Ok::<(), anyhow::Error>(());
                            }
                        }
                    }
                }
            }
        }
    };

    match timeout(Duration::from_secs(5), wait_loop).await {
        Ok(Ok(())) => {}
        Ok(Err(e)) => anyhow::bail!("Error reading message: {:?}", e),
        Err(_) => anyhow::bail!("Timed out waiting for undefined reference diagnostic"),
    }

    child.kill().await?;
    Ok(())
}

#[tokio::test]
async fn test_rename_flow() -> anyhow::Result<()> {
    // 1. Setup temp dir
    let temp_dir = tempfile::tempdir()?;
    let temp_path = temp_dir.path().to_owned();

    // 2. Locate binary
    let mut bin_path = None;
    let candidates = vec!["../../target/debug/ferrotexd", "target/debug/ferrotexd"];
    for candidate in candidates {
        let path = std::env::current_dir()?.join(candidate);
        if path.exists() {
            bin_path = Some(path);
            break;
        }
    }
    let final_bin_path = bin_path.ok_or_else(|| anyhow::anyhow!("ferrotexd binary not found"))?;

    // 3. Start server
    let mut child = Command::new(final_bin_path)
        .current_dir(&temp_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let stdin = child.stdin.as_mut().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);

    // 4. Initialize
    let init_msg = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "capabilities": {},
            "rootUri": format!("file://{}", temp_path.display()),
            "processId": std::process::id()
        }
    });
    send_msg(stdin, &init_msg).await?;
    read_msg(&mut reader).await?;

    let initialized_msg = json!({
        "jsonrpc": "2.0",
        "method": "initialized",
        "params": {}
    });
    send_msg(stdin, &initialized_msg).await?;

    // 5. Open document
    let doc_uri = format!("file://{}/main.tex", temp_path.display());
    // Line 0: \section{Intro}
    // Line 1: \label{oldName}
    // Line 2: See \ref{oldName}.
    let doc_text = "\\section{Intro}\n\\label{oldName}\nSee \\ref{oldName}.";
    let did_open = json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {
                "uri": doc_uri,
                "languageId": "latex",
                "version": 1,
                "text": doc_text
            }
        }
    });
    send_msg(stdin, &did_open).await?;

    // Wait for diagnostics (indexing complete)
    let mut indexed = false;
    for _ in 0..20 {
        let msg = timeout(Duration::from_secs(1), read_msg(&mut reader)).await??;
        if msg["method"] == "textDocument/publishDiagnostics" {
            indexed = true;
            break;
        }
    }
    assert!(indexed, "Timed out waiting for initial indexing");

    // 6. Request Rename: oldName -> newName
    // Position at line 1, char 8 (inside "oldName")
    let rename_req = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "textDocument/rename",
        "params": {
            "textDocument": { "uri": doc_uri },
            "position": { "line": 1, "character": 8 },
            "newName": "newName"
        }
    });
    send_msg(stdin, &rename_req).await?;

    // 7. Verify Edit
    let rename_res = loop {
        let msg = read_msg(&mut reader).await?;
        if let Some(id) = msg.get("id") {
            if id == 2 {
                break msg;
            }
        }
    };

    let workspace_edit = rename_res["result"]
        .as_object()
        .expect("Should have result");
    // Depending on client capabilities, server might return documentChanges or changes.
    // Our server implementation likely uses `changes`.
    // Let's check `changes` first.
    if let Some(changes) = workspace_edit.get("changes") {
        let changes_obj = changes.as_object().unwrap();
        let edits = changes_obj.get(&doc_uri).unwrap().as_array().unwrap();

        assert_eq!(edits.len(), 2, "Should have 2 edits (label and ref)");

        // Verify edits content
        for edit in edits {
            let new_text = edit["newText"].as_str().unwrap();
            assert_eq!(new_text, "newName");
        }
    } else if let Some(doc_changes) = workspace_edit.get("documentChanges") {
        // Fallback check if implementation changes
        let changes = doc_changes.as_array().unwrap();
        assert!(!changes.is_empty());
        // ... (simplified check)
    } else {
        panic!("WorkspaceEdit should contain changes or documentChanges");
    }

    child.kill().await?;
    Ok(())
}

async fn send_msg<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    msg: &serde_json::Value,
) -> anyhow::Result<()> {
    let s = msg.to_string();
    writer
        .write_all(format!("Content-Length: {}\r\n\r\n{}", s.len(), s).as_bytes())
        .await?;
    Ok(())
}

async fn read_msg<R: AsyncBufReadExt + Unpin>(reader: &mut R) -> anyhow::Result<serde_json::Value> {
    let mut content_length = 0;

    loop {
        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            anyhow::bail!("EOF while reading headers");
        }

        // Check for end of headers
        if line == "\r\n" || line == "\n" {
            break;
        }

        let line = line.trim();
        if let Some(rest) = line.strip_prefix("Content-Length: ") {
            content_length = rest.parse()?;
        }
    }

    if content_length == 0 {
        anyhow::bail!("No Content-Length header found");
    }

    let mut body = vec![0u8; content_length];
    reader.read_exact(&mut body).await?;

    let text = String::from_utf8(body)?;
    Ok(serde_json::from_str(&text)?)
}
