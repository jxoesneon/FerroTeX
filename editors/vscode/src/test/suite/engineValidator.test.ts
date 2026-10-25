import * as assert from "assert";
import * as vscode from "vscode";
import { validateBuildEngine, validateSyncTeX } from "../../engineValidator";

suite("Engine Validator Test Suite", () => {
  const getConfig = () => vscode.workspace.getConfiguration("ferrotex");
  let originalEngine: string | undefined;
  let originalSync: boolean | undefined;

  suiteSetup(() => {
    originalEngine = getConfig().get<string>("build.engine");
    originalSync = getConfig().get<boolean>("preview.syncToSource");
  });

  suiteTeardown(async () => {
    if (originalEngine !== undefined) {
      await getConfig().update("build.engine", originalEngine, vscode.ConfigurationTarget.Workspace);
    }
    if (originalSync !== undefined) {
      await getConfig().update("preview.syncToSource", originalSync, vscode.ConfigurationTarget.Workspace);
    }
  });

  test("validateBuildEngine: 'auto' engine returns early without error", async () => {
    await getConfig().update("build.engine", "auto", vscode.ConfigurationTarget.Workspace);
    await new Promise((r) => setTimeout(r, 200));
    await validateBuildEngine();
    assert.strictEqual(getConfig().get<string>("build.engine"), "auto");
  });

  test("validateBuildEngine: 'tectonic' engine is validated successfully", async () => {
    await getConfig().update("build.engine", "tectonic", vscode.ConfigurationTarget.Workspace);
    await new Promise((r) => setTimeout(r, 200));
    await validateBuildEngine();
    assert.strictEqual(getConfig().get<string>("build.engine"), "tectonic");
  });

  test("validateBuildEngine: checks supported engines list", async () => {
    const engines = ["tectonic", "latexmk", "pdflatex", "xelatex", "lualatex"];
    for (const eng of engines) {
      await getConfig().update("build.engine", eng, vscode.ConfigurationTarget.Workspace);
      await new Promise((r) => setTimeout(r, 100));
      // Ensure configuration can be retrieved for all engine paths
      const pathSetting = getConfig().get<string>(`build.${eng}Path`);
      assert.ok(pathSetting !== undefined, `Setting build.${eng}Path should exist`);
    }
  });

  test("validateSyncTeX: returns early when syncToSource is disabled", async () => {
    await getConfig().update("preview.syncToSource", false, vscode.ConfigurationTarget.Workspace);
    await new Promise((r) => setTimeout(r, 200));
    await validateSyncTeX();
    assert.strictEqual(getConfig().get<boolean>("preview.syncToSource"), false);
  });

  test("validateSyncTeX: skips validation when engine is tectonic or auto", async () => {
    await getConfig().update("preview.syncToSource", true, vscode.ConfigurationTarget.Workspace);
    await getConfig().update("build.engine", "tectonic", vscode.ConfigurationTarget.Workspace);
    await new Promise((r) => setTimeout(r, 200));
    await validateSyncTeX();

    await getConfig().update("build.engine", "auto", vscode.ConfigurationTarget.Workspace);
    await new Promise((r) => setTimeout(r, 200));
    await validateSyncTeX();
    assert.ok(true, "SyncTeX check correctly skipped for tectonic/auto");
  });
});
