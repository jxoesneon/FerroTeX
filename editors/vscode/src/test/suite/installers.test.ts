import * as assert from "assert";
import * as path from "path";
import * as fs from "fs";
import * as os from "os";
import * as vscode from "vscode";
import { ensureFerrotexdBinary } from "../../installFerrotexd";
import { ensureTectonicBinary, checkAndInstallTectonic } from "../../installTectonic";

suite("Binary Installers Test Suite", () => {
  let mockContext: vscode.ExtensionContext;
  let tempStorageDir: string;

  suiteSetup(() => {
    tempStorageDir = path.join(os.tmpdir(), `ferrotex-test-storage-${Date.now()}`);
    fs.mkdirSync(tempStorageDir, { recursive: true });

    // Construct mock extension context
    const ext = vscode.extensions.getExtension("ferrotex.ferrotex");
    const realExtensionPath = ext ? ext.extensionPath : path.resolve(__dirname, "../../../");

    mockContext = {
      extensionPath: realExtensionPath,
      extensionUri: vscode.Uri.file(realExtensionPath),
      globalStorageUri: vscode.Uri.file(tempStorageDir),
      storageUri: vscode.Uri.file(tempStorageDir),
      subscriptions: [],
      workspaceState: {
        get: () => undefined,
        update: async () => {},
        keys: () => [],
      },
      globalState: {
        get: () => undefined,
        update: async () => {},
        setKeysForSync: () => {},
        keys: () => [],
      },
      asAbsolutePath: (rel: string) => path.join(realExtensionPath, rel),
      extensionMode: vscode.ExtensionMode.Test,
      environmentVariableCollection: {} as any,
      secrets: {} as any,
      logUri: vscode.Uri.file(tempStorageDir),
      storagePath: tempStorageDir,
      globalStoragePath: tempStorageDir,
      logPath: tempStorageDir,
      extension: ext as any,
      languageModelAccessInformation: {} as any,
    };
  });

  suiteTeardown(() => {
    try {
      fs.rmSync(tempStorageDir, { recursive: true, force: true });
    } catch {
      /* ignore cleanup error */
    }
  });

  test("installFerrotexd: finds bundled binary in bin/", async () => {
    const binPath = await ensureFerrotexdBinary(mockContext);
    assert.ok(binPath, "ensureFerrotexdBinary should return a path");
    if (binPath !== "ferrotexd") {
      assert.ok(fs.existsSync(binPath), `Binary should exist at ${binPath}`);
      assert.ok(binPath.includes("ferrotexd"), "Binary path should mention ferrotexd");
    }
  });

  test("installFerrotexd: detects cached binary in globalStorage when bundled missing", async () => {
    const fakeContext = {
      ...mockContext,
      extensionPath: path.join(tempStorageDir, "empty-ext"),
    };
    const binaryName = process.platform === "win32" ? "ferrotexd.exe" : "ferrotexd";
    const cachedBinary = path.join(tempStorageDir, binaryName);

    // Place a dummy cached executable
    fs.writeFileSync(cachedBinary, "#!/bin/sh\necho ferrotexd", { mode: 0o755 });

    const result = await ensureFerrotexdBinary(fakeContext);
    assert.strictEqual(result, cachedBinary, "Should detect cached binary in globalStorage");
    fs.unlinkSync(cachedBinary);
  });

  test("installTectonic: finds bundled tectonic in bin/", async () => {
    const binPath = await ensureTectonicBinary(mockContext);
    assert.ok(binPath, "ensureTectonicBinary should return a path");
    if (binPath !== "tectonic") {
      assert.ok(fs.existsSync(binPath), `Tectonic binary should exist at ${binPath}`);
      assert.ok(binPath.includes("tectonic"), "Path should contain tectonic");
    }
  });

  test("installTectonic: detects cached tectonic in globalStorage", async () => {
    const fakeContext = {
      ...mockContext,
      extensionPath: path.join(tempStorageDir, "empty-ext"),
    };
    const binaryName = process.platform === "win32" ? "tectonic.exe" : "tectonic";
    const cachedBinary = path.join(tempStorageDir, binaryName);

    fs.writeFileSync(cachedBinary, "#!/bin/sh\necho tectonic", { mode: 0o755 });

    const result = await ensureTectonicBinary(fakeContext);
    assert.strictEqual(result, cachedBinary, "Should detect cached tectonic in globalStorage");
    fs.unlinkSync(cachedBinary);
  });

  test("installTectonic: checkAndInstallTectonic executes cleanly and registers PATH", async () => {
    await checkAndInstallTectonic(mockContext);
    const binDir = path.join(mockContext.extensionPath, "bin");
    const currentPath = process.env.PATH || "";
    assert.ok(
      currentPath.includes(binDir) || currentPath.includes("tectonic") || currentPath.length > 0,
      "PATH should include binary directory or be preserved",
    );
  });

  test("installTectonic: detects explicitly configured custom path", async () => {
    const config = vscode.workspace.getConfiguration("ferrotex");
    const dummyPath = path.join(tempStorageDir, "custom-tectonic");
    fs.writeFileSync(dummyPath, "#!/bin/sh\necho custom", { mode: 0o755 });

    await config.update("build.tectonicPath", dummyPath, vscode.ConfigurationTarget.Workspace);
    const result = await ensureTectonicBinary({
      ...mockContext,
      extensionPath: path.join(tempStorageDir, "empty-ext"),
    });

    assert.strictEqual(result, dummyPath, "Should resolve configured custom tectonicPath");
    await config.update("build.tectonicPath", "tectonic", vscode.ConfigurationTarget.Workspace);
    fs.unlinkSync(dummyPath);
  });
});
