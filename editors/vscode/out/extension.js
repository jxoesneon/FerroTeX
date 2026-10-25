"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const fs = __importStar(require("fs"));
const path = __importStar(require("path"));
const vscode = __importStar(require("vscode"));
const node_1 = require("vscode-languageclient/node");
const pdfPreview_1 = require("./pdfPreview");
const installTectonic_1 = require("./installTectonic");
const installFerrotexd_1 = require("./installFerrotexd");
const engineValidator_1 = require("./engineValidator");
const imagePaste_1 = require("./imagePaste");
let client;
/**
 * Resolves which TeX file to build for multi-file projects.
 * Priority: %!TEX root magic comment → .ferrotex.json config → VS Code setting → main.tex heuristic → current file
 */
async function resolveBuildTargetFromUri(uri) {
    const currentPath = uri.fsPath;
    const dir = path.dirname(currentPath);
    // 1. Check for %!TEX root magic comment
    try {
        const doc = await vscode.workspace.openTextDocument(uri);
        const text = doc.getText();
        const head = text.length > 1024 ? text.substring(0, 1024) : text;
        const magicMatch = head.match(/^%\s*!TEX\s+root\s*=\s*(.+)$/im);
        if (magicMatch) {
            const rootPath = magicMatch[1].trim();
            const resolved = path.resolve(dir, rootPath);
            if (fs.existsSync(resolved)) {
                return vscode.Uri.file(resolved).toString();
            }
        }
    }
    catch {
        // Can't read document, continue
    }
    // 2. .ferrotex.json config (walk up parent directories)
    let currentDir = dir;
    for (let i = 0; i < 20; i++) {
        const configPath = path.join(currentDir, ".ferrotex.json");
        if (fs.existsSync(configPath)) {
            try {
                const config = JSON.parse(fs.readFileSync(configPath, "utf-8"));
                if (config.mainFile) {
                    const mainPath = path.resolve(currentDir, config.mainFile);
                    if (fs.existsSync(mainPath)) {
                        return vscode.Uri.file(mainPath).toString();
                    }
                }
            }
            catch {
                // Invalid JSON, continue
            }
        }
        const parent = path.dirname(currentDir);
        if (parent === currentDir)
            break;
        currentDir = parent;
    }
    // 3. Check VS Code setting ferrotex.build.mainFile
    const config = vscode.workspace.getConfiguration("ferrotex");
    const mainFileSetting = config.get("build.mainFile", "");
    if (mainFileSetting) {
        const mainPath = path.resolve(dir, mainFileSetting);
        if (fs.existsSync(mainPath)) {
            return vscode.Uri.file(mainPath).toString();
        }
    }
    // 4. Check for main.tex heuristic
    const mainTexPath = path.join(dir, "main.tex");
    if (fs.existsSync(mainTexPath) && mainTexPath !== currentPath) {
        return vscode.Uri.file(mainTexPath).toString();
    }
    // 5. Fallback to current file
    return uri.toString();
}
async function resolveBuildTarget(editor) {
    return resolveBuildTargetFromUri(editor.document.uri);
}
async function activate(context) {
    const config = vscode.workspace.getConfiguration("ferrotex");
    let serverPath = config.get("serverPath");
    // Self-install Tectonic engine from jxoesneon/tectonic if not present
    await (0, installTectonic_1.checkAndInstallTectonic)(context);
    // Validate build engine when configuration changes
    context.subscriptions.push(vscode.workspace.onDidChangeConfiguration(async (e) => {
        if (e.affectsConfiguration("ferrotex.build.engine")) {
            await (0, engineValidator_1.validateBuildEngine)();
        }
        if (e.affectsConfiguration("ferrotex.preview.syncToSource")) {
            await (0, engineValidator_1.validateSyncTeX)();
        }
    }));
    // Validate engine and SyncTeX on startup
    (0, engineValidator_1.validateBuildEngine)();
    (0, engineValidator_1.validateSyncTeX)();
    // Resolve ferrotexd: bundled → globalStorage cache → PATH → auto-download
    if (!serverPath || serverPath === "ferrotexd") {
        const resolved = await (0, installFerrotexd_1.ensureFerrotexdBinary)(context);
        serverPath = resolved ?? "ferrotexd";
    }
    console.log("[FerroTeX] Server Path:", serverPath);
    const serverOptions = {
        run: { command: serverPath },
        debug: { command: serverPath },
    };
    const clientOptions = {
        documentSelector: [
            { scheme: "file", language: "latex" },
            { scheme: "file", language: "tex" },
        ],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher("**/.clientrc"),
        },
    };
    client = new node_1.LanguageClient("ferrotex", "FerroTeX Language Server", serverOptions, clientOptions);
    // UX-3: Image Paste Wizard
    if (vscode.languages.registerDocumentPasteEditProvider) {
        const selector = [
            { scheme: "file", language: "latex" },
            { scheme: "file", language: "tex" },
        ];
        context.subscriptions.push(vscode.languages.registerDocumentPasteEditProvider(selector, new imagePaste_1.ImagePasteProvider(), {
            pasteMimeTypes: ["image/png"],
            providedPasteEditKinds: [],
        }));
    }
    // EX-4: Integrated PDF Viewer
    const pdfProvider = new pdfPreview_1.PdfPreviewProvider(context.extensionUri, client);
    context.subscriptions.push(vscode.window.registerCustomEditorProvider(pdfPreview_1.PdfPreviewProvider.viewType, pdfProvider));
    // BO-1: Build Command
    context.subscriptions.push(vscode.commands.registerCommand("ferrotex.build", async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            vscode.window.showErrorMessage("Open a TeX file to build.");
            return;
        }
        // Auto-save before build
        await editor.document.save();
        const uri = await resolveBuildTarget(editor);
        try {
            await client.sendRequest("workspace/executeCommand", {
                command: "ferrotex.internal.build",
                arguments: [uri],
            });
        }
        catch (e) {
            vscode.window.showErrorMessage(`Build request failed: ${e}`);
        }
    }));
    // Package Installation Command
    context.subscriptions.push(vscode.commands.registerCommand("ferrotex.installPackage", async (packageName) => {
        const confirm = await vscode.window.showWarningMessage(`Install LaTeX package "${packageName}" using your package manager?`, { modal: true }, "Install");
        if (confirm === "Install") {
            await vscode.window.withProgress({
                location: vscode.ProgressLocation.Notification,
                title: `Installing ${packageName}...`,
                cancellable: false,
            }, async (progress) => {
                try {
                    const result = await client.sendRequest("workspace/executeCommand", {
                        command: "ferrotex.internal.installPackage",
                        arguments: [packageName],
                    });
                    if (result && result.success) {
                        vscode.window.showInformationMessage(`Successfully installed package "${packageName}"`);
                    }
                    else {
                        const error = result?.error || "Unknown error";
                        vscode.window.showErrorMessage(`Failed to install ${packageName}: ${error}`);
                    }
                }
                catch (e) {
                    vscode.window.showErrorMessage(`Installation error: ${e}`);
                }
            });
        }
    }));
    // Preview Button: Open PDF Preview
    context.subscriptions.push(vscode.commands.registerCommand("ferrotex.openPreview", async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            vscode.window.showErrorMessage("Open a TeX file to preview.");
            return;
        }
        const texUri = editor.document.uri;
        // Try to find the PDF
        // 1. Check build/ subdirectory (common output location)
        // 2. Check same directory as source
        const rootUriString = await resolveBuildTargetFromUri(texUri);
        const rootUri = vscode.Uri.parse(rootUriString);
        const baseName = path.basename(rootUri.fsPath, path.extname(rootUri.fsPath));
        const dirName = path.dirname(rootUri.fsPath);
        const possibleUris = [
            rootUri.with({ path: path.join(dirName, "build", `${baseName}.pdf`) }),
            rootUri.with({ path: path.join(dirName, `${baseName}.pdf`) }),
        ];
        let pdfUri = null;
        for (const candidate of possibleUris) {
            try {
                await vscode.workspace.fs.stat(candidate);
                pdfUri = candidate;
                break;
            }
            catch {
                // File doesn't exist, try next
            }
        }
        // If PDF doesn't exist, build it first
        if (!pdfUri) {
            vscode.window.showInformationMessage("PDF not found. Building...");
            await editor.document.save();
            try {
                await client.sendRequest("workspace/executeCommand", {
                    command: "ferrotex.internal.build",
                    arguments: [texUri.toString()],
                });
                // Wait a bit for build to complete and try again
                await new Promise((resolve) => setTimeout(resolve, 500));
                for (const candidate of possibleUris) {
                    try {
                        await vscode.workspace.fs.stat(candidate);
                        pdfUri = candidate;
                        break;
                    }
                    catch {
                        // Still doesn't exist
                    }
                }
            }
            catch (e) {
                vscode.window.showErrorMessage(`Build failed: ${e}`);
                return;
            }
        }
        if (!pdfUri) {
            vscode.window.showErrorMessage("Could not find or build PDF.");
            return;
        }
        // Open PDF in custom preview beside current editor
        try {
            await vscode.commands.executeCommand("vscode.openWith", pdfUri, "ferrotex.pdfPreview", vscode.ViewColumn.Beside);
        }
        catch (e) {
            vscode.window.showErrorMessage(`Failed to open preview: ${e}`);
        }
    }));
    // SX-2: SyncTeX Forward Search
    context.subscriptions.push(vscode.commands.registerCommand("ferrotex.syncToPdf", async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor)
            return;
        if (editor.document.languageId !== "latex" && editor.document.languageId !== "tex")
            return;
        const uri = editor.document.uri;
        const position = editor.selection.active;
        // Heuristic: PDF is in the same directory with .pdf extension
        // This is basic but works for standard builds.
        const pdfUri = uri.with({ path: uri.path.replace(/\.(tex|latex)$/i, ".pdf") });
        try {
            const result = await client.sendRequest("workspace/executeCommand", {
                command: "ferrotex.synctex_forward",
                arguments: [uri.toString(), position.line, position.character, pdfUri.toString()],
            });
            if (result) {
                // { page, x, y }
                pdfProvider.reveal(pdfUri, result.page, result.x, result.y);
            }
        }
        catch (e) {
            console.error("SyncTeX Forward failed:", e);
        }
    }));
    // Live Preview: Auto-build on save
    context.subscriptions.push(vscode.workspace.onDidSaveTextDocument(async (document) => {
        // Only trigger for LaTeX/TeX files
        if (document.languageId !== "latex" && document.languageId !== "tex") {
            return;
        }
        // Check if auto-build is enabled (add config option)
        const config = vscode.workspace.getConfiguration("ferrotex");
        const autoBuild = config.get("build.autoBuildOnSave", true);
        if (!autoBuild) {
            return;
        }
        // Trigger build
        const uri = await resolveBuildTargetFromUri(document.uri);
        try {
            await client.sendRequest("workspace/executeCommand", {
                command: "ferrotex.internal.build",
                arguments: [uri],
            });
        }
        catch (e) {
            console.error("[FerroTeX] Auto-build failed:", e);
        }
    }));
    // BO-2: Real-time Log Streaming
    const outputChannel = vscode.window.createOutputChannel("FerroTeX Build");
    context.subscriptions.push(outputChannel);
    // Register notification handler after client creation
    // We need to wait for client to be ready, or just register it.
    // Note: v8+ handling might differ, but onNotification is standard
    // To avoid race conditions, we can set it up immediately if possible, but usually wait for start.
    // Actually, standard practice is:
    client
        .start()
        .then(() => {
        console.log("[FerroTeX] Client started successfully.");
        client.onNotification("$/ferrotex/log", (params) => {
            outputChannel.append(params.message);
        });
    })
        .catch((e) => {
        console.error("[FerroTeX] Client start failed:", e);
        vscode.window.showErrorMessage(`FerroTeX Server Failed to Start: ${e.message || e}`);
    });
}
function deactivate() {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
//# sourceMappingURL=extension.js.map