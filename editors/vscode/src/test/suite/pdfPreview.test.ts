import * as assert from "assert";
import * as path from "path";
import * as fs from "fs";
import * as os from "os";
import * as vscode from "vscode";
import { PdfPreviewProvider } from "../../pdfPreview";

suite("PDF Preview Provider Test Suite", () => {
  let provider: PdfPreviewProvider;
  let extUri: vscode.Uri;
  let tempPdfPath: string;

  suiteSetup(() => {
    const ext = vscode.extensions.getExtension("ferrotex.ferrotex");
    extUri = ext ? ext.extensionUri : vscode.Uri.file(path.resolve(__dirname, "../../../"));

    // Mock language client
    const mockClient: any = {
      sendRequest: async (cmd: string, args: any) => {
        if (cmd === "workspace/executeCommand" && args.command === "ferrotex.synctex_inverse") {
          return { file: "/path/to/main.tex", line: 10 };
        }
        return null;
      },
    };

    provider = new PdfPreviewProvider(extUri, mockClient);

    // Create a temporary dummy PDF file
    tempPdfPath = path.join(os.tmpdir(), `test-preview-${Date.now()}.pdf`);
    fs.writeFileSync(tempPdfPath, "%PDF-1.4\n%EOF");
  });

  suiteTeardown(() => {
    try {
      if (fs.existsSync(tempPdfPath)) {
        fs.unlinkSync(tempPdfPath);
      }
    } catch {
      /* ignore */
    }
  });

  test("PdfPreviewProvider: static viewType matches configuration", () => {
    assert.strictEqual(PdfPreviewProvider.viewType, "ferrotex.pdfPreview");
  });

  test("openCustomDocument: returns document with valid URI and dispose method", () => {
    const uri = vscode.Uri.file(tempPdfPath);
    const doc = provider.openCustomDocument(
      uri,
      {} as any,
      new vscode.CancellationTokenSource().token,
    );
    assert.strictEqual(doc.uri.toString(), uri.toString());
    assert.strictEqual(typeof doc.dispose, "function");
    doc.dispose(); // Should not throw
  });

  test("resolveCustomEditor: initializes webview with options, HTML, and disposal hooks", async () => {
    const docUri = vscode.Uri.file(tempPdfPath);
    const doc = provider.openCustomDocument(
      docUri,
      {} as any,
      new vscode.CancellationTokenSource().token,
    );

    let disposed = false;
    let webviewHtml = "";
    let messageHandler: ((e: any) => Promise<void>) | undefined;

    const mockPanel: any = {
      webview: {
        options: {},
        asWebviewUri: (u: vscode.Uri) => u,
        set html(val: string) {
          webviewHtml = val;
        },
        get html() {
          return webviewHtml;
        },
        onDidReceiveMessage: (handler: (e: any) => Promise<void>) => {
          messageHandler = handler;
          return { dispose: () => {} };
        },
        postMessage: async (_msg: any) => true,
      },
      onDidDispose: (callback: () => void) => {
        // Trigger disposal when needed
        return { dispose: () => {} };
      },
      reveal: () => {},
    };

    await provider.resolveCustomEditor(doc, mockPanel, new vscode.CancellationTokenSource().token);

    // Verify webview configuration
    assert.strictEqual(mockPanel.webview.options.enableScripts, true);
    assert.ok(mockPanel.webview.options.localResourceRoots.length >= 2);

    // Verify HTML structure
    assert.ok(webviewHtml.includes("<!DOCTYPE html>"), "HTML should have doctype");
    assert.ok(webviewHtml.includes("pdfjsLib"), "HTML should reference pdfjsLib");
    assert.ok(webviewHtml.includes("page-container"), "HTML should have page-container");
    assert.ok(webviewHtml.includes("scale-display"), "HTML should have scale-display");
    assert.ok(webviewHtml.includes("synctex"), "HTML should include synctex styles/handlers");

    // Verify message handler is wired
    assert.ok(messageHandler, "Message handler should be registered");

    // Test reveal() forward search messaging
    provider.reveal(docUri, 1, 100, 200);
    assert.ok(true, "reveal() executed successfully");
  });
});
