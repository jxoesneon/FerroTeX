import * as vscode from "vscode";
import * as path from "path";

/**
 * Handles pasting of image data from the clipboard.
 *
 * Configurable via:
 * - `ferrotex.imagePaste.enabled`: Enable/disable.
 * - `ferrotex.imagePaste.defaultDirectory`: Target directory relative to source file (default: "figures").
 * - `ferrotex.imagePaste.filenamePattern`: Pattern for generating filenames (default: "image-{timestamp}").
 */
export class ImagePasteProvider implements vscode.DocumentPasteEditProvider {
  /**
   * Generates a unique filename based on the configured pattern.
   * Exposed for testing.
   */
  public generateFilename(pattern: string): string {
    const timestamp = new Date().toISOString().replace(/[:.]/g, "-");
    const uuid = Math.random().toString(36).substring(2, 9);

    // Simple replacement of variables
    let name = pattern.replace("{timestamp}", timestamp).replace("{uuid}", uuid);

    // Ensure extension
    if (!name.toLowerCase().endsWith(".png")) {
      name += ".png";
    }

    return name;
  }

  /**
   * Resolves the target relative path for LaTeX \includegraphics.
   * Exposed for testing.
   */
  public resolveRelativePath(docPath: string, imagePath: string): string {
    // If image is in a subdirectory of the doc, use relative path
    // e.g. doc: /a/b/main.tex, img: /a/b/figures/image.png -> figures/image.png
    const docDir = path.dirname(docPath);
    let relative = path.relative(docDir, imagePath);

    // Format for LaTeX (always forward slashes)
    relative = relative.split(path.sep).join("/");

    return relative;
  }

  async provideDocumentPasteEdits(
    document: vscode.TextDocument,
    ranges: readonly vscode.Range[],
    dataTransfer: vscode.DataTransfer,
    _context: vscode.DocumentPasteEditContext,
    token: vscode.CancellationToken,
  ): Promise<vscode.DocumentPasteEdit[] | undefined> {
    // Check if enabled
    const config = vscode.workspace.getConfiguration("ferrotex", document.uri);
    if (!config.get<boolean>("imagePaste.enabled", true)) {
      return undefined;
    }

    // Check for image data
    const item = dataTransfer.get("image/png");
    if (!item) {
      return undefined;
    }

    const file = item.asFile();
    if (!file) {
      return undefined;
    }

    // 1. Determine Target Path
    const defaultDir = config.get<string>("imagePaste.defaultDirectory", "figures");
    const pattern = config.get<string>("imagePaste.filenamePattern", "image-{timestamp}");

    const docDir = path.dirname(document.uri.fsPath);
    const targetDir = path.join(docDir, defaultDir);
    const filename = this.generateFilename(pattern);
    const targetPath = path.join(targetDir, filename);
    const targetUri = vscode.Uri.file(targetPath);

    // 2. Confirm with user (optional, but good UX to allow rename)
    // We'll skip the prompt for "speedy" paste as per "Wizard" spec,
    // but maybe standard behavior is just do it.
    // Let's stick to the automation: config-driven.

    // 3. Ensure directory exists
    try {
      await vscode.workspace.fs.createDirectory(vscode.Uri.file(targetDir));
    } catch (e) {
      vscode.window.showErrorMessage(`Failed to create directory: ${e}`);
      return undefined;
    }

    // 4. Write File
    try {
      const data = await file.data();
      await vscode.workspace.fs.writeFile(targetUri, data);
    } catch (e) {
      vscode.window.showErrorMessage(`Failed to save image: ${e}`);
      return undefined;
    }

    // 5. Generate Snippet
    const relativePath = this.resolveRelativePath(document.uri.fsPath, targetPath);
    // Remove extension for LaTeX cleanliness usually, but PNG is fine.
    // Standard is usually without extension if configured, but let's keep it simple.
    const snippetPath = relativePath.replace(/\.png$/i, "");
    const snippet = new vscode.SnippetString(`\\includegraphics{${snippetPath}}`);

    // 6. Return Edit
    const edit = new vscode.DocumentPasteEdit(snippet, "Paste Image", "insert" as any);
    return [edit];
  }
}
