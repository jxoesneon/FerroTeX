use serde_json::json;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, ReadHalf, WriteHalf, DuplexStream};
use tokio::time::{sleep, timeout};
use tower_lsp::lsp_types::Url;
use tower_lsp::{LspService, Server};

async fn setup_server() -> (BufReader<ReadHalf<DuplexStream>>, WriteHalf<DuplexStream>) {
    let (client_side, server_side) = tokio::io::duplex(1024 * 1024);
    let (service, socket) = LspService::new(|client| ferrotexd::Backend {
        client,
        documents: std::sync::Arc::new(dashmap::DashMap::new()),
        workspace: std::sync::Arc::new(ferrotexd::workspace::Workspace::new()),
        root_uri: std::sync::Arc::new(std::sync::Mutex::new(None)),
        syntax_diagnostics: std::sync::Arc::new(dashmap::DashMap::new()),
        package_manager: std::sync::Arc::new(std::sync::Mutex::new(ferrotex_core::package_manager::PackageManager::new())),
        package_index: std::sync::Arc::new(std::sync::Mutex::new(None)),
    });
    
    let (server_read, server_write) = tokio::io::split(server_side);
    tokio::spawn(Server::new(server_read, server_write, socket).serve(service));
    
    let (reader_half, writer_half) = tokio::io::split(client_side);
    (BufReader::new(reader_half), writer_half)
}

#[tokio::test]
async fn test_lsp_diagnostics_flow() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let temp_path = temp_dir.path().canonicalize()?;
    let (mut reader, mut writer) = setup_server().await;

    send_msg(&mut writer, &json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": { "capabilities": {}, "rootUri": Url::from_directory_path(&temp_path).unwrap(), "processId": std::process::id() }
    })).await?;
    read_msg(&mut reader).await?; 

    send_msg(&mut writer, &json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} })).await?;
    
    let tex_uri = Url::from_file_path(temp_path.join("test.tex")).unwrap();
    send_msg(&mut writer, &json!({
        "jsonrpc": "2.0", "method": "textDocument/didOpen",
        "params": { "textDocument": { "uri": tex_uri.clone(), "languageId": "latex", "version": 1, "text": "\\begin{document} \\end{document}" } }
    })).await?;

    sleep(Duration::from_secs(1)).await;
    let log_file = temp_path.join("test.log");
    tokio::fs::write(&log_file, "LaTeX Warning: Label `foo' multiply defined.\n").await?;

    let wait_loop = async {
        loop {
            let msg = read_msg(&mut reader).await?;
            if msg.get("method").and_then(|m| m.as_str()) == Some("textDocument/publishDiagnostics") {
                let params = &msg["params"];
                if params["uri"].as_str() == Some(tex_uri.as_str()) {
                    let diags = params["diagnostics"].as_array().expect("diagnostics array");
                    if diags.iter().any(|d| d["message"].as_str().unwrap().contains("Label `foo' multiply defined")) {
                        return Ok::<(), anyhow::Error>(());
                    }
                }
            }
        }
    };

    match timeout(Duration::from_secs(10), wait_loop).await {
        Ok(Ok(())) => {}
        Ok(Err(e)) => anyhow::bail!("Error reading message: {:?}", e),
        Err(_) => anyhow::bail!("Timed out waiting for log diagnostic"),
    }
    Ok(())
}

#[tokio::test]
async fn test_document_symbol_flow() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let temp_path = temp_dir.path().canonicalize()?;
    let (mut reader, mut writer) = setup_server().await;

    send_msg(&mut writer, &json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": { "capabilities": {}, "rootUri": Url::from_directory_path(&temp_path).unwrap() }
    })).await?;
    read_msg(&mut reader).await?;
    send_msg(&mut writer, &json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} })).await?;

    let doc_uri = Url::from_file_path(temp_path.join("main.tex")).unwrap();
    send_msg(&mut writer, &json!({
        "jsonrpc": "2.0", "method": "textDocument/didOpen",
        "params": { "textDocument": { "uri": doc_uri.clone(), "languageId": "latex", "version": 1, "text": "\\begin{document} \\end{document}" } }
    })).await?;

    send_msg(&mut writer, &json!({
        "jsonrpc": "2.0", "id": 2, "method": "textDocument/documentSymbol",
        "params": { "textDocument": { "uri": doc_uri } }
    })).await?;

    let syms = loop {
        let msg = read_msg(&mut reader).await?;
        if msg.get("id") == Some(&json!(2)) {
            break msg["result"].as_array().unwrap().clone();
        }
    };
    assert!(!syms.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_syntax_diagnostics_flow() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let temp_path = temp_dir.path().canonicalize()?;
    let (mut reader, mut writer) = setup_server().await;

    send_msg(&mut writer, &json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": { "capabilities": {}, "rootUri": Url::from_directory_path(&temp_path).unwrap() }
    })).await?;
    read_msg(&mut reader).await?;
    send_msg(&mut writer, &json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} })).await?;

    let doc_uri = Url::from_file_path(temp_path.join("broken.tex")).unwrap();
    send_msg(&mut writer, &json!({
        "jsonrpc": "2.0", "method": "textDocument/didOpen",
        "params": { "textDocument": { "uri": doc_uri.clone(), "languageId": "latex", "version": 1, "text": "{ \\cmd" } }
    })).await?;

    let mut found = false;
    for _ in 0..10 {
        let msg = read_msg(&mut reader).await?;
        if msg["method"] == "textDocument/publishDiagnostics" && msg["params"]["uri"] == doc_uri.as_str() {
            if msg["params"]["diagnostics"].as_array().unwrap().iter().any(|d| d["message"].as_str().unwrap().contains("Expected '}'")) {
                found = true;
                break;
            }
        }
    }
    assert!(found);
    Ok(())
}

#[tokio::test]
async fn test_label_features_flow() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let temp_path = temp_dir.path().canonicalize()?;
    let (mut reader, mut writer) = setup_server().await;

    send_msg(&mut writer, &json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": { "capabilities": {}, "rootUri": Url::from_directory_path(&temp_path).unwrap() }
    })).await?;
    read_msg(&mut reader).await?;
    send_msg(&mut writer, &json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} })).await?;

    let doc_uri = Url::from_file_path(temp_path.join("main.tex")).unwrap();
    send_msg(&mut writer, &json!({
        "jsonrpc": "2.0", "method": "textDocument/didOpen",
        "params": { "textDocument": { "uri": doc_uri.clone(), "languageId": "latex", "version": 1, "text": "\\section{Intro}\n\\label{sec:intro}\n\\ref{sec:intro}" } }
    })).await?;

    // Wait for indexing
    sleep(Duration::from_millis(500)).await;

    send_msg(&mut writer, &json!({
        "jsonrpc": "2.0", "id": 2, "method": "textDocument/definition",
        "params": { "textDocument": { "uri": doc_uri }, "position": { "line": 2, "character": 10 } }
    })).await?;

    let _ = loop {
        let msg = read_msg(&mut reader).await?;
        if msg.get("id") == Some(&json!(2)) { break msg; }
    };
    Ok(())
}

async fn send_msg<W: AsyncWriteExt + Unpin>(writer: &mut W, msg: &serde_json::Value) -> anyhow::Result<()> {
    let s = msg.to_string();
    writer.write_all(format!("Content-Length: {}\r\n\r\n{}", s.len(), s).as_bytes()).await?;
    Ok(())
}

async fn read_msg<R: AsyncBufReadExt + Unpin>(reader: &mut R) -> anyhow::Result<serde_json::Value> {
    let mut content_length = 0;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).await? == 0 { anyhow::bail!("EOF"); }
        if line == "\r\n" || line == "\n" { break; }
        if let Some(rest) = line.trim().strip_prefix("Content-Length: ") { content_length = rest.parse()?; }
    }
    if content_length == 0 { anyhow::bail!("No Content-Length"); }
    let mut body = vec![0u8; content_length];
    reader.read_exact(&mut body).await?;
    Ok(serde_json::from_str(&String::from_utf8(body)?)?)
}
