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
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.deactivate = exports.activate = void 0;
const path = __importStar(require("path"));
const vscode = __importStar(require("vscode"));
const node_1 = require("vscode-languageclient/node");
const pdfPreview_1 = require("./pdfPreview");
const installTectonic_1 = require("./installTectonic");
const engineValidator_1 = require("./engineValidator");
let client;
function activate(context) {
    const config = vscode.workspace.getConfiguration("ferrotex");
    let serverPath = config.get("serverPath");
    // UX-Upgrade: Frictionless Install
    (0, installTectonic_1.checkAndInstallTectonic)(context);
    // Validate build engine when configuration changes
    context.subscriptions.push(vscode.workspace.onDidChangeConfiguration(async (e) => {
        if (e.affectsConfiguration('ferrotex.build.engine')) {
            await (0, engineValidator_1.validateBuildEngine)();
        }
        if (e.affectsConfiguration('ferrotex.preview.syncToSource')) {
            await (0, engineValidator_1.validateSyncTeX)();
        }
    }));
    // Validate engine and SyncTeX on startup
    (0, engineValidator_1.validateBuildEngine)();
    (0, engineValidator_1.validateSyncTeX)();
    // If no path is configured, check for grouped/bundled binary first
    if (!serverPath || serverPath === "ferrotexd") {
        // Check bundled path: extensions/ferrotex/bin/ferrotexd
        const bundledPath = path.join(context.extensionPath, "bin", process.platform === "win32" ? "ferrotexd.exe" : "ferrotexd");
        const fs = require('fs');
        console.log("[FerroTeX] Checking bundled binary at:", bundledPath);
        if (fs.existsSync(bundledPath)) {
            serverPath = bundledPath;
            // UX-Upgrade: Ensure executable permissions on Linux/macOS
            if (process.platform !== "win32") {
                try {
                    fs.chmodSync(bundledPath, "755");
                    console.log("[FerroTeX] Set executable permissions for bundled binary.");
                }
                catch (err) {
                    console.error("[FerroTeX] Failed to set permissions:", err);
                }
            }
        }
        else {
            serverPath = "ferrotexd"; // Fallback to PATH
        }
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
        const metadata = {
            id: "ferrotex.imagePaste",
            pasteMimeTypes: ["image/png"],
        };
        context.subscriptions.push(vscode.languages.registerDocumentPasteEditProvider({ scheme: "file", language: "latex" }, new ImagePasteProvider(), metadata));
        context.subscriptions.push(vscode.languages.registerDocumentPasteEditProvider({ scheme: "file", language: "tex" }, new ImagePasteProvider(), metadata));
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
        const uri = editor.document.uri.toString();
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
        const baseName = path.basename(texUri.fsPath, path.extname(texUri.fsPath));
        const dirName = path.dirname(texUri.fsPath);
        const possiblePdfPaths = [
            path.join(dirName, "build", `${baseName}.pdf`),
            path.join(dirName, `${baseName}.pdf`)
        ];
        let pdfUri = null;
        for (const pdfPath of possiblePdfPaths) {
            try {
                await vscode.workspace.fs.stat(vscode.Uri.file(pdfPath));
                pdfUri = vscode.Uri.file(pdfPath);
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
                await new Promise(resolve => setTimeout(resolve, 500));
                for (const pdfPath of possiblePdfPaths) {
                    try {
                        await vscode.workspace.fs.stat(vscode.Uri.file(pdfPath));
                        pdfUri = vscode.Uri.file(pdfPath);
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
        const uri = document.uri.toString();
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
    client = new node_1.LanguageClient("ferrotex", "FerroTeX Language Server", serverOptions, clientOptions);
    // BO-2: Real-time Log Streaming
    const outputChannel = vscode.window.createOutputChannel("FerroTeX Build");
    context.subscriptions.push(outputChannel);
    // Register notification handler after client creation
    // We need to wait for client to be ready, or just register it.
    // Note: v8+ handling might differ, but onNotification is standard
    // To avoid race conditions, we can set it up immediately if possible, but usually wait for start.
    // Actually, standard practice is:
    client.start().then(() => {
        console.log("[FerroTeX] Client started successfully.");
        client.onNotification("$/ferrotex/log", (params) => {
            outputChannel.append(params.message);
        });
    }).catch(e => {
        console.error("[FerroTeX] Client start failed:", e);
        vscode.window.showErrorMessage(`FerroTeX Server Failed to Start: ${e.message || e}`);
    });
}
exports.activate = activate;
function deactivate() {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
exports.deactivate = deactivate;
/**
 * Handles pasting of image data from the clipboard.
 *
 * - Prompts the user for a filename.
 * - Saves the image file relative to the document.
 * - Inserts an `\includegraphics{...}` snippet.
 */
class ImagePasteProvider {
    async provideDocumentPasteEdits(document, ranges, dataTransfer, _context, token) {
        const item = dataTransfer.get("image/png");
        if (!item) {
            return undefined;
        }
        const file = item.asFile();
        if (!file) {
            return undefined;
        }
        // 1. Ask for filename
        const name = await vscode.window.showInputBox({
            prompt: "Enter filename for pasted image (e.g., plot.png)",
            value: "image.png",
            ignoreFocusOut: true,
        });
        if (!name) {
            return undefined;
        }
        // 2. Save file relative to document
        const uri = vscode.Uri.joinPath(document.uri, "..", name);
        try {
            const data = await file.data();
            await vscode.workspace.fs.writeFile(uri, data);
        }
        catch (e) {
            vscode.window.showErrorMessage(`Failed to save image: ${e}`);
            return undefined;
        }
        // 3. Insert snippet
        const snippet = new vscode.SnippetString(`\\includegraphics{${name}}`);
        // Create edit
        // We replace the range (which is usually empty "paste" position, or selection)
        // Pass 'kind' as any to bypass private constructor issue/missing static
        const edit = new vscode.DocumentPasteEdit(snippet, "Insert Image", "insert");
        return [edit];
    }
}
//# sourceMappingURL=extension.js.map