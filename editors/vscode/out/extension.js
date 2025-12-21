"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.deactivate = exports.activate = void 0;
const vscode = require("vscode");
const node_1 = require("vscode-languageclient/node");
const pdfPreview_1 = require("./pdfPreview");
let client;
function activate(context) {
    const config = vscode.workspace.getConfiguration("ferrotex");
    const serverPath = config.get("serverPath") || "ferrotexd";
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
            id: 'ferrotex.imagePaste',
            pasteMimeTypes: ['image/png']
        };
        context.subscriptions.push(vscode.languages.registerDocumentPasteEditProvider({ scheme: "file", language: "latex" }, new ImagePasteProvider(), metadata));
        context.subscriptions.push(vscode.languages.registerDocumentPasteEditProvider({ scheme: "file", language: "tex" }, new ImagePasteProvider(), metadata));
    }
    // EX-4: Integrated PDF Viewer
    // EX-4: Integrated PDF Viewer
    const pdfProvider = new pdfPreview_1.PdfPreviewProvider(context.extensionUri, client);
    context.subscriptions.push(vscode.window.registerCustomEditorProvider(pdfPreview_1.PdfPreviewProvider.viewType, pdfProvider));
    // SX-2: SyncTeX Forward Search
    context.subscriptions.push(vscode.commands.registerCommand('ferrotex.syncToPdf', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor)
            return;
        if (editor.document.languageId !== 'latex' && editor.document.languageId !== 'tex')
            return;
        const uri = editor.document.uri;
        const position = editor.selection.active;
        // Heuristic: PDF is in the same directory with .pdf extension
        // This is basic but works for standard builds.
        const pdfUri = uri.with({ path: uri.path.replace(/\.(tex|latex)$/i, '.pdf') });
        try {
            const result = await client.sendRequest('workspace/executeCommand', {
                command: 'ferrotex.synctex_forward',
                arguments: [
                    uri.toString(),
                    position.line,
                    position.character,
                    pdfUri.toString()
                ]
            });
            if (result) {
                // { page, x, y }
                pdfProvider.reveal(pdfUri, result.page, result.x, result.y);
            }
        }
        catch (e) {
            console.error('SyncTeX Forward failed:', e);
        }
    }));
    client.start();
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
        const item = dataTransfer.get('image/png');
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
            ignoreFocusOut: true
        });
        if (!name) {
            return undefined;
        }
        // 2. Save file relative to document
        const uri = vscode.Uri.joinPath(document.uri, '..', name);
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