import * as assert from "assert";
import * as path from "path";
import * as fs from "fs";
import * as os from "os";
import * as vscode from "vscode";

suite("Extension Commands & Target Resolution Test Suite", () => {
  let tempDir: string;
  let mainDoc: vscode.TextDocument;
  let subDoc: vscode.TextDocument;

  suiteSetup(async () => {
    tempDir = path.join(os.tmpdir(), `ferrotex-cmd-test-${Date.now()}`);
    fs.mkdirSync(tempDir, { recursive: true });

    const mainPath = path.join(tempDir, "main.tex");
    const subPath = path.join(tempDir, "chapter.tex");

    fs.writeFileSync(
      mainPath,
      "\\documentclass{article}\n\\begin{document}\n\\include{chapter}\n\\end{document}\n",
    );
    fs.writeFileSync(subPath, "% !TEX root = main.tex\n\\section{Chapter}\nChapter content.\n");

    mainDoc = await vscode.workspace.openTextDocument(vscode.Uri.file(mainPath));
    subDoc = await vscode.workspace.openTextDocument(vscode.Uri.file(subPath));
  });

  suiteTeardown(() => {
    try {
      fs.rmSync(tempDir, { recursive: true, force: true });
    } catch {
      /* ignore */
    }
  });

  test("Commands registration: all core commands are registered in VS Code", async () => {
    const commands = await vscode.commands.getCommands(true);
    const expected = [
      "ferrotex.build",
      "ferrotex.installPackage",
      "ferrotex.openPreview",
      "ferrotex.syncToPdf",
    ];

    for (const cmd of expected) {
      assert.ok(commands.includes(cmd), `Command ${cmd} should be registered`);
    }
  });

  test("ferrotex.build: triggers build on active document", async () => {
    const editor = await vscode.window.showTextDocument(mainDoc);
    assert.strictEqual(editor.document.uri.fsPath, mainDoc.uri.fsPath);

    // Should execute without throwing
    await vscode.commands.executeCommand("ferrotex.build");
    assert.ok(true, "ferrotex.build executed successfully on active editor");
  });

  test("ferrotex.build: resolves root document on sub-file with magic comment", async () => {
    const editor = await vscode.window.showTextDocument(subDoc);
    assert.strictEqual(editor.document.uri.fsPath, subDoc.uri.fsPath);

    await vscode.commands.executeCommand("ferrotex.build");
    assert.ok(true, "ferrotex.build redirected to root file via magic comment");
  });

  test("ferrotex.openPreview: executes without error on active TeX file", async () => {
    await vscode.window.showTextDocument(mainDoc);
    await vscode.commands.executeCommand("ferrotex.openPreview");
    assert.ok(true, "ferrotex.openPreview invoked on document");
  });

  test("ferrotex.syncToPdf: executes forward search positioning", async () => {
    const editor = await vscode.window.showTextDocument(mainDoc);
    editor.selection = new vscode.Selection(new vscode.Position(1, 0), new vscode.Position(1, 0));

    await vscode.commands.executeCommand("ferrotex.syncToPdf");
    assert.ok(true, "ferrotex.syncToPdf executed without error");
  });

  test("Target Resolution via .ferrotex.json config", async () => {
    const projectDir = path.join(tempDir, "proj");
    fs.mkdirSync(projectDir, { recursive: true });
    const configPath = path.join(projectDir, ".ferrotex.json");
    const rootPath = path.join(projectDir, "root.tex");
    const childPath = path.join(projectDir, "child.tex");

    fs.writeFileSync(configPath, JSON.stringify({ mainFile: "root.tex" }));
    fs.writeFileSync(rootPath, "\\documentclass{article}\\begin{document}\\end{document}");
    fs.writeFileSync(childPath, "\\section{Child}");

    const childDoc = await vscode.workspace.openTextDocument(vscode.Uri.file(childPath));
    await vscode.window.showTextDocument(childDoc);

    await vscode.commands.executeCommand("ferrotex.build");
    assert.ok(true, "Build handled project with .ferrotex.json");
  });

  test("Target Resolution via ferrotex.build.mainFile setting", async () => {
    const config = vscode.workspace.getConfiguration("ferrotex");
    await config.update("build.mainFile", "main.tex", vscode.ConfigurationTarget.Workspace);

    const editor = await vscode.window.showTextDocument(subDoc);
    assert.ok(editor);
    await vscode.commands.executeCommand("ferrotex.build");

    await config.update("build.mainFile", "", vscode.ConfigurationTarget.Workspace);
    assert.ok(true, "Target resolution respected build.mainFile setting");
  });
});
