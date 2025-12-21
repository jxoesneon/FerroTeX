import * as vscode from "vscode";
import * as child_process from "child_process";

/**
 * Validates that the selected build engine is available
 * Shows helpful messages if engine is not found
 */
export async function validateBuildEngine() {
  const config = vscode.workspace.getConfiguration("ferrotex");
  const selectedEngine = config.get<string>("build.engine", "auto");

  if (selectedEngine === "auto") {
    // Auto mode will try Tectonic first, then fallback
    return;
  }

  // Map engines to their path settings
  const enginePaths: { [key: string]: string } = {
    tectonic: config.get<string>("build.tectonicPath", "tectonic"),
    latexmk: config.get<string>("build.latexmkPath", "latexmk"),
    pdflatex: config.get<string>("build.pdflatexPath", "pdflatex"),
    xelatex: config.get<string>("build.xelatexPath", "xelatex"),
    lualatex: config.get<string>("build.lualatexPath", "lualatex"),
  };

  const enginePath = enginePaths[selectedEngine];
  if (!enginePath) {
    return; // Unknown engine
  }

  // Check if engine exists
  const isAvailable = await checkEngineAvailable(enginePath);

  if (!isAvailable) {
    const action = await vscode.window.showWarningMessage(
      `Build engine "${selectedEngine}" not found. FerroTeX won't be able to build documents.`,
      "Specify Custom Path",
      "Install Guide",
      "Switch to Tectonic",
    );

    if (action === "Specify Custom Path") {
      // Open settings to the specific engine path setting
      vscode.commands.executeCommand(
        "workbench.action.openSettings",
        `@id:ferrotex.build.${selectedEngine}Path`,
      );
    } else if (action === "Install Guide") {
      const installGuides: { [key: string]: string } = {
        latexmk: "https://www.tug.org/texlive/",
        pdflatex: "https://www.tug.org/texlive/",
        xelatex: "https://www.tug.org/texlive/",
        lualatex: "https://www.tug.org/texlive/",
      };

      const url = installGuides[selectedEngine];
      if (url) {
        vscode.env.openExternal(vscode.Uri.parse(url));
      }
    } else if (action === "Switch to Tectonic") {
      await config.update("build.engine", "tectonic", vscode.ConfigurationTarget.Global);
      vscode.window.showInformationMessage("Build engine switched to Tectonic (zero-config).");
    }
  }
}

/**
 * Checks if a build engine command is available
 */
async function checkEngineAvailable(command: string): Promise<boolean> {
  return new Promise((resolve) => {
    // Try to run --version or --help to check if command exists
    child_process.exec(`${command} --version`, (error) => {
      resolve(!error);
    });
  });
}

/**
 * Validates that SyncTeX is available for forward/backward sync
 * Only warns if user is NOT using Tectonic (which doesn't include synctex CLI)
 */
export async function validateSyncTeX() {
  const config = vscode.workspace.getConfiguration("ferrotex");
  const syncEnabled = config.get<boolean>("preview.syncToSource", true);
  const buildEngine = config.get<string>("build.engine", "auto");

  if (!syncEnabled) {
    return; // User disabled it
  }

  // Skip validation if using Tectonic or auto mode (which defaults to Tectonic)
  // Tectonic doesn't include synctex CLI but generates .synctex.gz files
  // The backend will handle missing synctex gracefully
  if (buildEngine === "tectonic" || buildEngine === "auto") {
    return;
  }

  // Check if synctex command exists (for other engines)
  const isAvailable = await checkEngineAvailable("synctex");

  if (!isAvailable) {
    const action = await vscode.window.showWarningMessage(
      "SyncTeX not found. Source ↔ PDF navigation won't work. Install a full TeX distribution (TeX Live, MiKTeX, or MacTeX).",
      "Disable SyncTeX",
      "Install Guide",
      "Ignore",
    );

    if (action === "Disable SyncTeX") {
      await config.update("preview.syncToSource", false, vscode.ConfigurationTarget.Global);
      vscode.window.showInformationMessage("SyncTeX integration disabled.");
    } else if (action === "Install Guide") {
      vscode.env.openExternal(vscode.Uri.parse("https://www.tug.org/texlive/"));
    }
  }
}
