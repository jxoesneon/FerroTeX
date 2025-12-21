"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.deactivate = exports.activate = void 0;
const vscode = require("vscode");
const node_1 = require("vscode-languageclient/node");
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