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
  if (platform === "linux" && arch === "x64") return "x86_64-unknown-linux-musl";
  if (platform === "linux" && arch === "arm64") return "aarch64-unknown-linux-musl";
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
  const url = `https://api.github.com/repos/${GITHUB_REPO}/releases/tags/continuous`;
  try {
    const data = await httpsGet(url);
    return JSON.parse(data.toString("utf8")) as GitHubRelease;
  } catch {
    const fallbackUrl = `https://api.github.com/repos/${GITHUB_REPO}/releases`;
    const data = await httpsGet(fallbackUrl);
    const releases = JSON.parse(data.toString("utf8")) as GitHubRelease[];
    if (releases && releases.length > 0) {
      return releases[0];
    }
    throw new Error("No releases found for jxoesneon/tectonic");
  }
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

function hasCommand(cmd: string): boolean {
  try {
    const checkCmd = process.platform === "win32" ? "where" : "which";
    const result = spawnSync(checkCmd, [cmd]);
    return result.status === 0;
  } catch {
    return false;
  }
}

function registerTectonicPath(binPath: string): void {
  const dir = path.dirname(binPath);
  const currentPath = process.env.PATH || "";
  const parts = currentPath.split(path.delimiter);
  if (!parts.includes(dir)) {
    process.env.PATH = `${dir}${path.delimiter}${currentPath}`;
  }
}

/**
 * Ensures our customized Tectonic binary is available, self-installing it
 * directly from jxoesneon/tectonic if not present.
 */
export async function ensureTectonicBinary(
  context: vscode.ExtensionContext,
): Promise<string | undefined> {
  // 1. Check bundled binary inside extension/bin/
  const bundledPath = path.join(context.extensionPath, "bin", BINARY_NAME);
  if (fs.existsSync(bundledPath)) {
    if (process.platform !== "win32") {
      try {
        fs.chmodSync(bundledPath, "755");
      } catch {
        /* ignore */
      }
    }
    console.log("[FerroTeX] Using bundled Tectonic:", bundledPath);
    registerTectonicPath(bundledPath);
    return bundledPath;
  }

  // 2. Check cached binary in globalStorage
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
    console.log("[FerroTeX] Using cached Tectonic:", storedPath);
    registerTectonicPath(storedPath);
    return storedPath;
  }

  // 3. Check explicitly configured path
  const config = vscode.workspace.getConfiguration("ferrotex");
  const configuredPath = config.get<string>("build.tectonicPath");
  if (configuredPath && configuredPath !== "tectonic" && fs.existsSync(configuredPath)) {
    console.log("[FerroTeX] Using configured Tectonic:", configuredPath);
    registerTectonicPath(configuredPath);
    return configuredPath;
  }

  // 4. Check system PATH
  if (hasCommand("tectonic")) {
    console.log("[FerroTeX] Using Tectonic from PATH");
    return "tectonic";
  }

  // 5. Self-install from jxoesneon/tectonic GitHub continuous release
  const target = getPlatformTarget();
  if (!target) {
    vscode.window.showErrorMessage(
      `FerroTeX: Unsupported platform (${process.platform}/${process.arch}) for Tectonic self-installation.`,
    );
    return undefined;
  }

  return vscode.window.withProgress(
    {
      location: vscode.ProgressLocation.Notification,
      title: "FerroTeX: Self-installing Tectonic engine (jxoesneon/tectonic)...",
      cancellable: false,
    },
    async (progress) => {
      try {
        if (!fs.existsSync(storageDir)) {
          fs.mkdirSync(storageDir, { recursive: true });
        }

        progress.report({ message: "Locating release asset..." });
        let downloadUrl = "";
        const ext = getArchiveExtension();

        try {
          const release = await fetchLatestRelease();
          const asset = release.assets.find((a) => a.name.includes(target) && a.name.endsWith(ext));
          if (asset) {
            downloadUrl = asset.browser_download_url;
          }
        } catch (e) {
          console.warn(
            "[FerroTeX] Could not query GitHub release API, using direct download URL",
            e,
          );
        }

        if (!downloadUrl) {
          downloadUrl = `https://github.com/${GITHUB_REPO}/releases/download/continuous/tectonic-0.17.3-${target}${ext}`;
        }

        progress.report({ message: `Downloading Tectonic (${target})...` });
        const archiveData = await httpsGet(downloadUrl);

        const tmpArchive = path.join(os.tmpdir(), `tectonic-${target}${ext}`);
        fs.writeFileSync(tmpArchive, archiveData);

        progress.report({ message: "Extracting Tectonic..." });
        if (process.platform === "win32") {
          extractZip(tmpArchive, storageDir);
        } else {
          extractTarGz(tmpArchive, storageDir);
        }
        try {
          fs.unlinkSync(tmpArchive);
        } catch {
          /* ignore */
        }

        if (!fs.existsSync(storedPath)) {
          throw new Error(`Extraction succeeded but ${BINARY_NAME} not found in ${storageDir}`);
        }

        if (process.platform !== "win32") {
          fs.chmodSync(storedPath, "755");
        }

        // Also copy to extension bin/ if that directory exists
        const extBinDir = path.join(context.extensionPath, "bin");
        if (fs.existsSync(extBinDir)) {
          const extBinPath = path.join(extBinDir, BINARY_NAME);
          try {
            fs.copyFileSync(storedPath, extBinPath);
            if (process.platform !== "win32") {
              fs.chmodSync(extBinPath, "755");
            }
          } catch {
            /* ignore */
          }
        }

        registerTectonicPath(storedPath);
        vscode.window.showInformationMessage(
          "FerroTeX: Tectonic TeX engine self-installed successfully.",
        );
        console.log("[FerroTeX] Self-installed Tectonic to:", storedPath);
        return storedPath;
      } catch (err: unknown) {
        const msg = err instanceof Error ? err.message : String(err);
        vscode.window.showErrorMessage(`FerroTeX: Failed to self-install Tectonic — ${msg}`);
        console.error("[FerroTeX] Tectonic self-install failed:", err);
        return undefined;
      }
    },
  );
}

/**
 * Checks and self-installs Tectonic if needed upon extension activation.
 */
export async function checkAndInstallTectonic(context: vscode.ExtensionContext): Promise<void> {
  const binary = await ensureTectonicBinary(context);
  if (binary) {
    registerTectonicPath(binary);
  }
}
