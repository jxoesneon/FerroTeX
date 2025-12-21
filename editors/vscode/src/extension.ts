import * as path from "path";
import * as vscode from "vscode";
import { LanguageClient, LanguageClientOptions, ServerOptions } from "vscode-languageclient/node";

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
  const config = vscode.workspace.getConfiguration("ferrotex");
  const serverPath = config.get<string>("serverPath") || "ferrotexd";

  const serverOptions: ServerOptions = {
    run: { command: serverPath },
    debug: { command: serverPath },
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [
      { scheme: "file", language: "latex" },
      { scheme: "file", language: "tex" },
    ],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher("**/.clientrc"),
    },
  };

  client = new LanguageClient("ferrotex", "FerroTeX Language Server", serverOptions, clientOptions);

  // UX-3: Image Paste Wizard
  if (vscode.languages.registerDocumentPasteEditProvider) {
      const metadata = { 
          id: 'ferrotex.imagePaste', 
          pasteMimeTypes: ['image/png']
      };
      context.subscriptions.push(
          vscode.languages.registerDocumentPasteEditProvider(
              { scheme: "file", language: "latex" },
              new ImagePasteProvider(),
              metadata as any
          )
      );
      context.subscriptions.push(
        vscode.languages.registerDocumentPasteEditProvider(
            { scheme: "file", language: "tex" },
            new ImagePasteProvider(),
            metadata as any
        )
    );
  }

  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}

/**
 * Handles pasting of image data from the clipboard.
 * 
 * - Prompts the user for a filename.
 * - Saves the image file relative to the document.
 * - Inserts an `\includegraphics{...}` snippet.
 */
class ImagePasteProvider implements vscode.DocumentPasteEditProvider {
    async provideDocumentPasteEdits(
        document: vscode.TextDocument,
        ranges: readonly vscode.Range[],
        dataTransfer: vscode.DataTransfer,
        _context: vscode.DocumentPasteEditContext,
        token: vscode.CancellationToken
    ): Promise<vscode.DocumentPasteEdit[] | undefined> {
        const item = dataTransfer.get('image/png');
        if (!item) { return undefined; }
        
        const file = item.asFile();
        if (!file) { return undefined; }
        
        // 1. Ask for filename
        const name = await vscode.window.showInputBox({ 
            prompt: "Enter filename for pasted image (e.g., plot.png)",
            value: "image.png",
            ignoreFocusOut: true
        });
        if (!name) { return undefined; }
        
        // 2. Save file relative to document
        const uri = vscode.Uri.joinPath(document.uri, '..', name);
        
        try {
            const data = await file.data();
            await vscode.workspace.fs.writeFile(uri, data);
        } catch (e) {
            vscode.window.showErrorMessage(`Failed to save image: ${e}`);
            return undefined;
        }
        
        // 3. Insert snippet
        const snippet = new vscode.SnippetString(`\\includegraphics{${name}}`);
        
        // Create edit
        // We replace the range (which is usually empty "paste" position, or selection)
        // Pass 'kind' as any to bypass private constructor issue/missing static
        const edit = new vscode.DocumentPasteEdit(snippet, "Insert Image", "insert" as any);
        
        return [edit];
    }
}
