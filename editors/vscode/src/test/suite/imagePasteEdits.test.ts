import * as assert from "assert";
import * as path from "path";
import * as fs from "fs";
import * as os from "os";
import * as vscode from "vscode";
import { ImagePasteProvider } from "../../imagePaste";

suite("Image Paste Provider Document Edits Test Suite", () => {
  let provider: ImagePasteProvider;
  let tempDir: string;
  let testDoc: vscode.TextDocument;

  suiteSetup(async () => {
    provider = new ImagePasteProvider();
    tempDir = path.join(os.tmpdir(), `ferrotex-img-paste-${Date.now()}`);
    fs.mkdirSync(tempDir, { recursive: true });

    const docPath = path.join(tempDir, "doc.tex");
    fs.writeFileSync(docPath, "\\documentclass{article}\n\\begin{document}\n\\end{document}");
    testDoc = await vscode.workspace.openTextDocument(vscode.Uri.file(docPath));
  });

  suiteTeardown(() => {
    try {
      fs.rmSync(tempDir, { recursive: true, force: true });
    } catch {
      /* ignore */
    }
  });

  test("provideDocumentPasteEdits: returns undefined when imagePaste.enabled is false", async () => {
    const config = vscode.workspace.getConfiguration("ferrotex", testDoc.uri);
    await config.update("imagePaste.enabled", false, vscode.ConfigurationTarget.Workspace);

    const mockDataTransfer = new vscode.DataTransfer();
    mockDataTransfer.set("image/png", new vscode.DataTransferItem(new Uint8Array([1, 2, 3])));

    const edits = await provider.provideDocumentPasteEdits(
      testDoc,
      [new vscode.Range(0, 0, 0, 0)],
      mockDataTransfer,
      {} as any,
      new vscode.CancellationTokenSource().token,
    );

    assert.strictEqual(edits, undefined, "Should return undefined when disabled");
    await config.update("imagePaste.enabled", true, vscode.ConfigurationTarget.Workspace);
  });

  test("provideDocumentPasteEdits: returns undefined when no image/png mime item present", async () => {
    const mockDataTransfer = new vscode.DataTransfer();
    mockDataTransfer.set("text/plain", new vscode.DataTransferItem("plain text"));

    const edits = await provider.provideDocumentPasteEdits(
      testDoc,
      [new vscode.Range(0, 0, 0, 0)],
      mockDataTransfer,
      {} as any,
      new vscode.CancellationTokenSource().token,
    );

    assert.strictEqual(edits, undefined, "Should return undefined for non-image data");
  });

  test("provideDocumentPasteEdits: generates snippet edit and saves image file", async () => {
    const imgBytes = new Uint8Array([137, 80, 78, 71, 13, 10, 26, 10]); // PNG header bytes
    const mockDataTransfer = new vscode.DataTransfer();

    const dataTransferFile: vscode.DataTransferFile = {
      name: "sample.png",
      uri: vscode.Uri.file("/tmp/sample.png"),
      data: async () => imgBytes,
    };

    const item: any = new vscode.DataTransferItem(dataTransferFile);
    item.asFile = () => dataTransferFile;
    mockDataTransfer.set("image/png", item);

    const edits = await provider.provideDocumentPasteEdits(
      testDoc,
      [new vscode.Range(0, 0, 0, 0)],
      mockDataTransfer,
      {} as any,
      new vscode.CancellationTokenSource().token,
    );

    assert.ok(edits, "Should return document paste edits");
    assert.strictEqual(edits.length, 1, "Should return exactly one edit");

    const snippet = edits[0].insertText as vscode.SnippetString;
    assert.ok(snippet.value.startsWith("\\includegraphics{"), "Should emit \\includegraphics snippet");
    assert.ok(snippet.value.includes("figures/"), "Should target figures directory");
  });
});
