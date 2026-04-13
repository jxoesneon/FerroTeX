import * as vscode from "vscode";
import * as path from "path";
import * as fs from "fs";
import * as https from "https";
import * as os from "os";
import { spawnSync } from "child_process";

const GITHUB_REPO = "jxoesneon/tectonic";
const BINARY_NAME = process.platform === "win32" ? "tectonic.exe" : "tectonic";

interface GitHubAsset {
  name: string;
  browser_download_url: string;
}

interface GitHubRelease {
  tag_name: string;
  assets: GitHubAsset[];
}

function getPlatformTarget(): string | undefined {
  const { platform, arch } = process;
  if (platform === "win32" && arch === "x64") return "x86_64-pc-windows-msvc";
  if (platform === "linux" && arch === "x64") return "x86_64-unknown-linux-gnu";
  if (platform === "darwin" && arch === "x64") return "x86_64-apple-darwin";
  if (platform === "darwin" && arch === "arm64") return "aarch64-apple-darwin";
  return undefined;
}

function getArchiveExtension(): string {
  return process.platform === "win32" ? ".zip" : ".tar.gz";
}

function httpsGet(url: string): Promise<Buffer> {
  return new Promise((resolve, reject) => {
    const request = (u: string) => {
      https
        .get(u, { headers: { "User-Agent": "ferrotex-vscode" } }, (res) => {
          if (res.statusCode === 301 || res.statusCode === 302) {
            request(res.headers.location!);
            return;
          }
          if (res.statusCode !== 200) {
            reject(new Error(`HTTP ${res.statusCode} for ${u}`));
            return;
          }
          const chunks: Buffer[] = [];
          res.on("data", (chunk) => chunks.push(chunk));
          res.on("end", () => resolve(Buffer.concat(chunks)));
          res.on("error", reject);
        })
        .on("error", reject);
    };
    request(url);
  });
}

async function fetchLatestRelease(): Promise<GitHubRelease> {
  const url = `https://api.github.com/repos/${GITHUB_REPO}/releases/latest`;
  const data = await httpsGet(url);
  return JSON.parse(data.toString("utf8")) as GitHubRelease;
}

function extractTarGz(archivePath: string, destDir: string): void {
  const result = spawnSync("tar", ["-xzf", archivePath, "-C", destDir], { encoding: "utf8" });
  if (result.status !== 0) {
    throw new Error(`tar extraction failed: ${result.stderr ?? result.error?.message ?? ""}`);
  }
}

function extractZip(archivePath: string, destDir: string): void {
  const result = spawnSync(
    "powershell",
    ["-Command", "Expand-Archive", "-Path", archivePath, "-DestinationPath", destDir, "-Force"],
    { encoding: "utf8" },
  );
  if (result.status !== 0) {
    throw new Error(`Expand-Archive failed: ${result.stderr ?? result.error?.message ?? ""}`);
  }
}

/**
 * Checks if a command exists on the system PATH
 */
function hasCommand(cmd: string): boolean {
  try {
    const checkCmd = process.platform === "win32" ? "where" : "which";
    spawnSync(checkCmd, [cmd], { stdio: "ignore" });
    return true;
  } catch {
    return false;
  }
}

/**
 * Ensures the custom Tectonic binary is available.
 * Checks: bundled → globalStorage cache → system PATH → auto-download
 * Returns the directory containing the binary (for PATH injection), or undefined.
 */
export async function ensureTectonicBinary(
  context: vscode.ExtensionContext,
): Promise<string | undefined> {
  const config = vscode.workspace.getConfiguration("ferrotex");
  const autoInstall = config.get<boolean>("build.autoInstallTectonic", true);

  // 1. Check bundled binary (shipped inside the .vsix)
  const bundledPath = path.join(context.extensionPath, "bin", BINARY_NAME);
  if (fs.existsSync(bundledPath)) {
    if (process.platform !== "win32") {
      try {
        fs.chmodSync(bundledPath, "755");
      } catch {
        /* ignore */
      }
    }
    console.log("[FerroTeX] Using bundled tectonic:", bundledPath);
    return path.dirname(bundledPath);
  }

  // 2. Check previously downloaded binary in globalStorage
  const storageDir = context.globalStorageUri.fsPath;
  const storedPath = path.join(storageDir, BINARY_NAME);
  if (fs.existsSync(storedPath)) {
    if (process.platform !== "win32") {
      try {
        fs.chmodSync(storedPath, "755");
      } catch {
        /* ignore */
      }
    }
    console.log("[FerroTeX] Using cached tectonic:", storedPath);
    return storageDir;
  }

  // 3. Check system PATH
  if (hasCommand("tectonic")) {
    console.log("[FerroTeX] Using tectonic from PATH");
    return undefined; // Signal to use PATH
  }

  // 4. Auto-download from GitHub releases (if enabled)
  if (!autoInstall) {
    return undefined;
  }

  const target = getPlatformTarget();
  if (!target) {
    vscode.window.showWarningMessage(
      `FerroTeX: Unsupported platform (${process.platform}/${process.arch}) for auto-install. ` +
        "Please install Tectonic manually.",
    );
    return undefined;
  }

  const proceed = await vscode.window.showInformationMessage(
    "FerroTeX: Tectonic PDF engine not found. Download the custom fork automatically?",
    { modal: false },
    "Download",
    "Not Now",
  );
  if (proceed !== "Download") {
    return undefined;
  }

  return vscode.window.withProgress(
    {
      location: vscode.ProgressLocation.Notification,
      title: "FerroTeX: Downloading Tectonic...",
      cancellable: false,
    },
    async (progress) => {
      try {
        progress.report({ message: "Fetching release info..." });
        const release = await fetchLatestRelease();
        const ext = getArchiveExtension();
        const assetName = `tectonic-${target}${ext}`;
        const asset = release.assets.find((a) => a.name === assetName);

        if (!asset) {
          throw new Error(
            `No release asset found for your platform (${assetName}). ` +
              "Please install Tectonic manually.",
          );
        }

        progress.report({ message: `Downloading ${release.tag_name}...` });
        const archiveData = await httpsGet(asset.browser_download_url);

        // Write archive to a temp file
        if (!fs.existsSync(storageDir)) {
          fs.mkdirSync(storageDir, { recursive: true });
        }
        const tmpArchive = path.join(os.tmpdir(), assetName);
        fs.writeFileSync(tmpArchive, archiveData);

        progress.report({ message: "Extracting..." });
        if (process.platform === "win32") {
          extractZip(tmpArchive, storageDir);
        } else {
          extractTarGz(tmpArchive, storageDir);
        }
        fs.unlinkSync(tmpArchive);

        // The archive should contain the binary directly
        if (!fs.existsSync(storedPath)) {
          throw new Error(
            `Extraction succeeded but ${BINARY_NAME} not found in ${storageDir}. ` +
              "Archive layout may have changed.",
          );
        }

        if (process.platform !== "win32") {
          fs.chmodSync(storedPath, "755");
        }

        vscode.window.showInformationMessage(
          `FerroTeX: Tectonic ${release.tag_name} installed successfully.`,
        );
        console.log("[FerroTeX] Downloaded tectonic to:", storedPath);
        return storageDir;
      } catch (err: unknown) {
        const msg = err instanceof Error ? err.message : String(err);
        vscode.window.showErrorMessage(`FerroTeX: Failed to download Tectonic — ${msg}`);
        console.error("[FerroTeX] Tectonic auto-install failed:", err);
        return undefined;
      }
    },
  );
}

/**
 * Legacy function: checks for TeX engines and offers installation options.
 * Now primarily delegates to ensureTectonicBinary for the custom fork.
 */
export async function checkAndInstallTectonic(context: vscode.ExtensionContext): Promise<void> {
  // First, try to ensure our custom Tectonic fork is available
  const tectonicDir = await ensureTectonicBinary(context);

  // If we have it (bundled, cached, or auto-downloaded), we're done
  if (tectonicDir) {
    return;
  }

  // Check if system has any TeX engine available
  if (hasCommand("latexmk") || hasCommand("tectonic")) {
    return;
  }

  // No engine found and auto-install either failed or was declined
  const config = vscode.workspace.getConfiguration("ferrotex");
  const autoInstall = config.get<boolean>("build.autoInstallTectonic", true);

  if (!autoInstall) {
    // User disabled auto-install, just warn
    vscode.window.showWarningMessage(
      "FerroTeX: No TeX engine found. Enable 'ferrotex.build.autoInstallTectonic' for automatic installation.",
    );
    return;
  }

  // Auto-install was attempted but failed or was declined - offer fallback options
  const selection = await vscode.window.showWarningMessage(
    "FerroTeX: No TeX engine found. PDF builds will fail.",
    "Install Guide",
    "Dismiss",
  );

  if (selection === "Install Guide") {
    vscode.env.openExternal(
      vscode.Uri.parse("https://tectonic-typesetting.github.io/en-US/install.html"),
    );
  }
}
