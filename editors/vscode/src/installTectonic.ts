import * as vscode from "vscode";
import * as path from "path";
import * as fs from "fs";
import { exec } from "child_process";

export async function checkAndInstallTectonic(context: vscode.ExtensionContext): Promise<void> {
  const config = vscode.workspace.getConfiguration("ferrotex");

  // Check system for latexmk or tectonic
  if (hasCommand("latexmk") || hasCommand("tectonic")) {
    return;
  }

  // Also check if we previously installed it (or set it up) to avoid spamming
  // In a real implementation we would persist state.

  // If on macOS and brew exists, offer to install
  if (process.platform === "darwin" && hasCommand("brew")) {
    const selection = await vscode.window.showWarningMessage(
      "FerroTeX: No TeX engine found. Install Tectonic via Homebrew?",
      "Install Tectonic",
      "See Instructions",
    );

    if (selection === "Install Tectonic") {
      const term = vscode.window.createTerminal("FerroTeX Install");
      term.show();
      term.sendText("brew install tectonic");
      // We can't easily wait for terminal command completion without complex tasks API,
      // but this is a good start.
      vscode.window.showInformationMessage(
        "Installing Tectonic... Please restart VS Code buffer after completion.",
      );
    } else if (selection === "See Instructions") {
      vscode.env.openExternal(
        vscode.Uri.parse("https://tectonic-typesetting.github.io/en-US/install.html"),
      );
    }
  } else {
    // Generic fallback
    const selection = await vscode.window.showErrorMessage(
      "FerroTeX: No TeX engine found (latexmk/tectonic). PDF builds will fail.",
      "Install Tectonic (Web)",
      "Dismiss",
    );
    if (selection === "Install Tectonic (Web)") {
      vscode.env.openExternal(
        vscode.Uri.parse("https://tectonic-typesetting.github.io/en-US/install.html"),
      );
    }
  }
}

function hasCommand(cmd: string): boolean {
  try {
    const checkCmd = process.platform === "win32" ? "where" : "which";
    require("child_process").execSync(`${checkCmd} ${cmd}`);
    return true;
  } catch {
    return false;
  }
}
