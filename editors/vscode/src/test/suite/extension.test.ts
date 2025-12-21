import * as assert from "assert";
import * as vscode from "vscode";

suite("Extension Test Suite", () => {
  vscode.window.showInformationMessage("Start all tests.");

  test("Extension should be present", () => {
    assert.ok(vscode.extensions.getExtension("ferrotex.ferrotex"));
  });

  test("Extension should activate", async () => {
    const ext = vscode.extensions.getExtension("ferrotex.ferrotex");
    if (ext) {
      await ext.activate();
      assert.strictEqual(ext.isActive, true);
    }
  });

  test("Commands should be registered", async () => {
    const commands = await vscode.commands.getCommands(true);
    assert.ok(commands.includes("ferrotex.build"));
    assert.ok(commands.includes("ferrotex.syncToPdf"));
  });

  test("Configuration should have default serverPath", () => {
    const config = vscode.workspace.getConfiguration("ferrotex");
    const serverPath = config.get<string>("serverPath");
    assert.strictEqual(serverPath, "ferrotexd");
  });
});
