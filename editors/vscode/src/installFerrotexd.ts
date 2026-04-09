import * as vscode from "vscode";
import * as path from "path";
import * as fs from "fs";
import * as https from "https";
import * as os from "os";
import { execSync } from "child_process";

const GITHUB_REPO = "jxoesneon/FerroTeX";
const BINARY_NAME = process.platform === "win32" ? "ferrotexd.exe" : "ferrotexd";

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
  execSync(`tar -xzf "${archivePath}" -C "${destDir}"`);
}

function extractZip(archivePath: string, destDir: string): void {
  execSync(
    `powershell -Command "Expand-Archive -Path '${archivePath}' -DestinationPath '${destDir}' -Force"`,
  );
}

/**
 * Ensures ferrotexd is available, downloading it from GitHub releases if needed.
 * Returns the path to the binary, or undefined if provisioning failed.
 */
export async function ensureFerrotexdBinary(
  context: vscode.ExtensionContext,
): Promise<string | undefined> {
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
    console.log("[FerroTeX] Using bundled ferrotexd:", bundledPath);
    return bundledPath;
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
    console.log("[FerroTeX] Using cached ferrotexd:", storedPath);
    return storedPath;
  }

  // 3. Check system PATH
  try {
    const checkCmd = process.platform === "win32" ? "where ferrotexd" : "which ferrotexd";
    execSync(checkCmd, { stdio: "ignore" });
    console.log("[FerroTeX] Using ferrotexd from PATH");
    return "ferrotexd";
  } catch {
    // Not on PATH — proceed to download
  }

  // 4. Auto-download from GitHub releases
  const target = getPlatformTarget();
  if (!target) {
    vscode.window.showErrorMessage(
      `FerroTeX: Unsupported platform (${process.platform}/${process.arch}). ` +
        "Please install ferrotexd manually and set ferrotex.serverPath.",
    );
    return undefined;
  }

  const proceed = await vscode.window.showInformationMessage(
    "FerroTeX: The language server (ferrotexd) is not installed. Download it automatically?",
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
      title: "FerroTeX: Downloading language server...",
      cancellable: false,
    },
    async (progress) => {
      try {
        progress.report({ message: "Fetching release info..." });
        const release = await fetchLatestRelease();
        const ext = getArchiveExtension();
        const assetName = `ferrotexd-${target}${ext}`;
        const asset = release.assets.find((a) => a.name === assetName);

        if (!asset) {
          throw new Error(
            `No release asset found for your platform (${assetName}). ` +
              "Please install ferrotexd manually.",
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
          `FerroTeX: Language server ${release.tag_name} installed successfully.`,
        );
        console.log("[FerroTeX] Downloaded ferrotexd to:", storedPath);
        return storedPath;
      } catch (err: unknown) {
        const msg = err instanceof Error ? err.message : String(err);
        vscode.window.showErrorMessage(`FerroTeX: Failed to download ferrotexd — ${msg}`);
        console.error("[FerroTeX] Auto-install failed:", err);
        return undefined;
      }
    },
  );
}
