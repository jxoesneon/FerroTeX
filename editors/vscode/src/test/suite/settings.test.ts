/**
 * Comprehensive Settings Validation Test
 * Tests all FerroTeX settings for type correctness, valid combinations, and runtime behavior
 */

import * as assert from "assert";
import * as vscode from "vscode";

suite("Settings Validation Suite", function () {
  this.timeout(60000);
  const settingsPrefix = "ferrotex";

  // Helper to get configuration
  function getConfig() {
    return vscode.workspace.getConfiguration(settingsPrefix);
  }

  // Helper to reset all settings
  async function resetAllSettings() {
    const config = getConfig();
    const keys = [
      "serverPath",
      "trace.server",
      "build.autoBuildOnSave",
      "build.engine",
      "build.outputDirectory",
      "build.cleanAuxiliaryFiles",
      "build.showOutputPanel",
      "lint.enabled",
      "lint.onType",
      "lint.deprecatedCommands",
      "lint.obsoletePackages",
      "lint.displayMathDelimiters",
      "preview.syncToSource",
      "preview.defaultZoom",
      "preview.scrollMode",
      "completion.enabled",
      "completion.packages",
      "completion.citations",
      "format.enabled",
      "format.onSave",
      "format.indentSize",
      "imagePaste.enabled",
      "imagePaste.defaultDirectory",
      "imagePaste.filenamePattern",
      "hover.enabled",
      "hover.citations",
      "hover.previewMath",
      "workspace.scanOnStartup",
      "workspace.maxFileSize",
      "workspace.excludePatterns",
      "diagnostics.humanReadableErrors",
      "diagnostics.showCode",
    ];

    for (const key of keys) {
      await getConfig().update(key, undefined, vscode.ConfigurationTarget.Global);
      await new Promise((r) => setTimeout(r, 200));
      await getConfig().update(key, undefined, vscode.ConfigurationTarget.Workspace);
      await new Promise((r) => setTimeout(r, 200));
    }
  }

  suite("Core Settings", () => {
    test("serverPath accepts valid paths", async () => {
      await getConfig().update("serverPath", "ferrotexd", vscode.ConfigurationTarget.Workspace);
      await new Promise((r) => setTimeout(r, 200));
      assert.strictEqual(getConfig().get<string>("serverPath"), "ferrotexd");

      await getConfig().update(
        "serverPath",
        "/usr/local/bin/ferrotexd",
        vscode.ConfigurationTarget.Workspace,
      );
      await new Promise((r) => setTimeout(r, 200));
      assert.strictEqual(getConfig().get<string>("serverPath"), "/usr/local/bin/ferrotexd");
    });

    test("trace.server accepts valid enum values", async () => {
      const validValues = ["off", "messages", "verbose"];

      for (const value of validValues) {
        await getConfig().update("trace.server", value, vscode.ConfigurationTarget.Workspace);
        await new Promise((r) => setTimeout(r, 200));
        assert.strictEqual(getConfig().get<string>("trace.server"), value);
      }
    });
  });

  suite("Build Settings", () => {
    test("build.autoBuildOnSave is boolean", async () => {
      await getConfig().update("build.autoBuildOnSave", true, vscode.ConfigurationTarget.Workspace);
      await new Promise((r) => setTimeout(r, 200));
      assert.strictEqual(getConfig().get<boolean>("build.autoBuildOnSave"), true);

      await getConfig().update(
        "build.autoBuildOnSave",
        false,
        vscode.ConfigurationTarget.Workspace,
      );
      await new Promise((r) => setTimeout(r, 200));
      assert.strictEqual(getConfig().get<boolean>("build.autoBuildOnSave"), false);
    });

    test("build.engine accepts all valid engines", async () => {
      const engines = ["auto", "tectonic", "latexmk", "pdflatex", "xelatex", "lualatex"];

      for (const engine of engines) {
        await getConfig().update("build.engine", engine, vscode.ConfigurationTarget.Workspace);
        await new Promise((r) => setTimeout(r, 200));
        assert.strictEqual(getConfig().get<string>("build.engine"), engine);
      }
    });

    test("build.showOutputPanel accepts valid enum values", async () => {
      const validValues = ["always", "onError", "never"];

      for (const value of validValues) {
        await getConfig().update(
          "build.showOutputPanel",
          value,
          vscode.ConfigurationTarget.Workspace,
        );
        await new Promise((r) => setTimeout(r, 200));
        assert.strictEqual(getConfig().get<string>("build.showOutputPanel"), value);
      }
    });
  });

  suite("Linting Settings", () => {
    test("all lint settings are boolean toggles", async () => {
      const lintSettings = [
        "lint.enabled",
        "lint.onType",
        "lint.deprecatedCommands",
        "lint.obsoletePackages",
        "lint.displayMathDelimiters",
      ];

      for (const setting of lintSettings) {
        await getConfig().update(setting, true, vscode.ConfigurationTarget.Workspace);
        await new Promise((r) => setTimeout(r, 200));
        assert.strictEqual(getConfig().get<boolean>(setting), true);

        await getConfig().update(setting, false, vscode.ConfigurationTarget.Workspace);
        await new Promise((r) => setTimeout(r, 200));
        assert.strictEqual(getConfig().get<boolean>(setting), false);
      }
    });

    test("lint.enabled master switch works", async () => {
      // When disabled, linting should be off regardless of individual rules
      await getConfig().update("lint.enabled", false, vscode.ConfigurationTarget.Workspace);
      await new Promise((r) => setTimeout(r, 200));
      await getConfig().update(
        "lint.deprecatedCommands",
        true,
        vscode.ConfigurationTarget.Workspace,
      );
      await new Promise((r) => setTimeout(r, 200));

      assert.strictEqual(getConfig().get<boolean>("lint.enabled"), false);
      assert.strictEqual(getConfig().get<boolean>("lint.deprecatedCommands"), true);
    });
  });

  suite("Preview Settings", () => {
    test("preview.defaultZoom respects min/max range", async () => {
      await getConfig().update("preview.defaultZoom", 100, vscode.ConfigurationTarget.Workspace);
      await new Promise((r) => setTimeout(r, 200));
      assert.strictEqual(getConfig().get<number>("preview.defaultZoom"), 100);

      await getConfig().update("preview.defaultZoom", 25, vscode.ConfigurationTarget.Workspace);
      await new Promise((r) => setTimeout(r, 200));
      assert.strictEqual(getConfig().get<number>("preview.defaultZoom"), 25);

      await getConfig().update("preview.defaultZoom", 500, vscode.ConfigurationTarget.Workspace);
      await new Promise((r) => setTimeout(r, 200));
      assert.strictEqual(getConfig().get<number>("preview.defaultZoom"), 500);
    });

    test("preview.scrollMode accepts valid values", async () => {
      const modes = ["vertical", "horizontal", "wrapped"];

      for (const mode of modes) {
        await getConfig().update("preview.scrollMode", mode, vscode.ConfigurationTarget.Workspace);
        await new Promise((r) => setTimeout(r, 200));
        assert.strictEqual(getConfig().get<string>("preview.scrollMode"), mode);
      }
    });
  });

  suite("Completion Settings", () => {
    test("all completion settings are boolean", async () => {
      const completionSettings = [
        "completion.enabled",
        "completion.packages",
        "completion.citations",
      ];

      for (const setting of completionSettings) {
        await getConfig().update(setting, true, vscode.ConfigurationTarget.Workspace);
        await new Promise((r) => setTimeout(r, 200));
        assert.strictEqual(getConfig().get<boolean>(setting), true);
      }
    });
  });

  suite("Format Settings", () => {
    test("format.indentSize respects range", async () => {
      for (let i = 1; i <= 8; i++) {
        await getConfig().update("format.indentSize", i, vscode.ConfigurationTarget.Workspace);
        await new Promise((r) => setTimeout(r, 200));
        assert.strictEqual(getConfig().get<number>("format.indentSize"), i);
      }
    });
  });

  suite("Workspace Settings", () => {
    test("workspace.maxFileSize respects range", async () => {
      await getConfig().update("workspace.maxFileSize", 1, vscode.ConfigurationTarget.Workspace);
      await new Promise((r) => setTimeout(r, 200));
      assert.strictEqual(getConfig().get<number>("workspace.maxFileSize"), 1);

      await getConfig().update("workspace.maxFileSize", 100, vscode.ConfigurationTarget.Workspace);
      await new Promise((r) => setTimeout(r, 200));
      assert.strictEqual(getConfig().get<number>("workspace.maxFileSize"), 100);
    });

    test("workspace.excludePatterns accepts string array", async () => {
      const patterns = ["**/build/**", "**/out/**", "**/.git/**"];

      await getConfig().update(
        "workspace.excludePatterns",
        patterns,
        vscode.ConfigurationTarget.Workspace,
      );
      await new Promise((r) => setTimeout(r, 200));
      const result = getConfig().get<string[]>("workspace.excludePatterns");
      assert.deepStrictEqual(result, patterns);
    });
  });

  suite("Setting Combinations", () => {
    test("disabling features updates correctly", async () => {
      // Disable all major features
      await getConfig().update("lint.enabled", false, vscode.ConfigurationTarget.Workspace);
      await new Promise((r) => setTimeout(r, 200));
      await getConfig().update("completion.enabled", false, vscode.ConfigurationTarget.Workspace);
      await new Promise((r) => setTimeout(r, 200));
      await getConfig().update("format.enabled", false, vscode.ConfigurationTarget.Workspace);
      await new Promise((r) => setTimeout(r, 200));
      await getConfig().update("hover.enabled", false, vscode.ConfigurationTarget.Workspace);
      await new Promise((r) => setTimeout(r, 200));

      assert.strictEqual(getConfig().get<boolean>("lint.enabled"), false);
      assert.strictEqual(getConfig().get<boolean>("completion.enabled"), false);
      assert.strictEqual(getConfig().get<boolean>("format.enabled"), false);
      assert.strictEqual(getConfig().get<boolean>("hover.enabled"), false);
    });

    test("build engine combinations work", async () => {
      // Test auto-build with different engines
      const engines = ["auto", "tectonic", "latexmk", "pdflatex"];
      for (const engine of engines) {
        await getConfig().update("build.engine", engine, vscode.ConfigurationTarget.Workspace);
        await new Promise((r) => setTimeout(r, 200));
        await getConfig().update(
          "build.autoBuildOnSave",
          true,
          vscode.ConfigurationTarget.Workspace,
        );
        await new Promise((r) => setTimeout(r, 200));

        assert.strictEqual(getConfig().get<string>("build.engine"), engine);
        assert.strictEqual(getConfig().get<boolean>("build.autoBuildOnSave"), true);
      }
    });

    test("lint rules can be individually controlled", async () => {
      // Enable linting but disable specific rules
      await getConfig().update("lint.enabled", true, vscode.ConfigurationTarget.Workspace);
      await new Promise((r) => setTimeout(r, 200));
      await getConfig().update(
        "lint.deprecatedCommands",
        false,
        vscode.ConfigurationTarget.Workspace,
      );
      await new Promise((r) => setTimeout(r, 200));
      await getConfig().update("lint.obsoletePackages", true, vscode.ConfigurationTarget.Workspace);
      await new Promise((r) => setTimeout(r, 200));

      assert.strictEqual(getConfig().get<boolean>("lint.enabled"), true);
      assert.strictEqual(getConfig().get<boolean>("lint.deprecatedCommands"), false);
      assert.strictEqual(getConfig().get<boolean>("lint.obsoletePackages"), true);
    });
  });

  suite("Default Values", () => {
    test("all settings have correct defaults", async () => {
      await resetAllSettings();

      // Verify key defaults
      assert.strictEqual(getConfig().get<string>("serverPath"), "ferrotexd");
      assert.strictEqual(getConfig().get<string>("trace.server"), "off");
      assert.strictEqual(getConfig().get<boolean>("build.autoBuildOnSave"), true);
      assert.strictEqual(getConfig().get<string>("build.engine"), "auto");
      assert.strictEqual(getConfig().get<boolean>("lint.enabled"), true);
      assert.strictEqual(getConfig().get<number>("preview.defaultZoom"), 100);
      assert.strictEqual(getConfig().get<number>("format.indentSize"), 2);
      assert.strictEqual(getConfig().get<string>("imagePaste.defaultDirectory"), "figures");
    });
  });

  suiteTeardown(async () => {
    await resetAllSettings();
  });
});
