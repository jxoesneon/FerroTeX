import * as vscode from "vscode";
import { LanguageClient } from "vscode-languageclient/node";

/**
 * integrated PDF Viewer provider for FerroTeX.
 *
 * Uses `pdfjs-dist` to render PDFs in a Webview.
 * Supports SyncTeX (Forward and Inverse).
 */
export class PdfPreviewProvider implements vscode.CustomReadonlyEditorProvider {
  public static readonly viewType = "ferrotex.pdfPreview";
  private readonly panels = new Map<string, vscode.WebviewPanel>();

  constructor(
    private readonly extensionUri: vscode.Uri,
    private readonly client: LanguageClient,
  ) {}

  openCustomDocument(
    uri: vscode.Uri,
    _openContext: vscode.CustomDocumentOpenContext,
    _token: vscode.CancellationToken,
  ): vscode.CustomDocument {
    return { uri, dispose: () => {} };
  }

  async resolveCustomEditor(
    document: vscode.CustomDocument,
    webviewPanel: vscode.WebviewPanel,
    _token: vscode.CancellationToken,
  ): Promise<void> {
    this.panels.set(document.uri.toString(), webviewPanel);

    webviewPanel.onDidDispose(() => {
      this.panels.delete(document.uri.toString());
    });

    webviewPanel.webview.options = {
      enableScripts: true,
      localResourceRoots: [
        vscode.Uri.joinPath(this.extensionUri, "dist"),
        vscode.Uri.file(document.uri.path).with({
          path: document.uri.path.substring(0, document.uri.path.lastIndexOf("/") + 1),
        }),
      ],
    };

    webviewPanel.webview.html = this.getHtmlForWebview(webviewPanel.webview, document.uri);

    webviewPanel.webview.onDidReceiveMessage(async (e) => {
      if (e.command === "synctex_inverse") {
        const args = [document.uri.toString(), e.page, e.x, e.y];
        const result = (await this.client.sendRequest("workspace/executeCommand", {
          command: "ferrotex.synctex_inverse",
          arguments: args,
        })) as any;

        if (result) {
          const fileUri = vscode.Uri.file(result.file);
          const doc = await vscode.workspace.openTextDocument(fileUri);
          const editor = await vscode.window.showTextDocument(doc, vscode.ViewColumn.One);
          const pos = new vscode.Position(result.line, 0);
          editor.selection = new vscode.Selection(pos, pos);
          editor.revealRange(new vscode.Range(pos, pos), vscode.TextEditorRevealType.InCenter);
        }
      }
    });
  }

  /**
   * Reveals a specific location in the PDF.
   * Used for SyncTeX Forward Search.
   */
  public reveal(pdfUri: vscode.Uri, page: number, x: number, y: number) {
    const panel = this.panels.get(pdfUri.toString());
    if (panel) {
      panel.reveal();
      panel.webview.postMessage({ command: "synctex_forward", page, x, y });
    }
  }

  private getHtmlForWebview(webview: vscode.Webview, pdfUri: vscode.Uri): string {
    const pdfJsUri = webview.asWebviewUri(
      vscode.Uri.joinPath(this.extensionUri, "dist", "pdfjs", "pdf.mjs"),
    );
    const pdfWorkerUri = webview.asWebviewUri(
      vscode.Uri.joinPath(this.extensionUri, "dist", "pdfjs", "pdf.worker.mjs"),
    );

    const pdfContentUri = webview.asWebviewUri(pdfUri);

    return `<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>PDF Preview</title>
            <style>
                body { margin: 0; padding: 0; background-color: #525659; display: flex; flex-direction: column; height: 100vh; overflow: hidden; }
                :root {
                    --bg-color: #1e1e1e;
                    --toolbar-bg: rgba(30, 30, 30, 0.8);
                    --accent: #3b82f6;
                    --text-color: #e5e7eb;
                }
                body { 
                    margin: 0; 
                    padding: 0; 
                    background-color: var(--bg-color); 
                    display: flex; 
                    flex-direction: column; 
                    height: 100vh; 
                    overflow: hidden; 
                    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
                }
                #toolbar {
                    position: absolute;
                    bottom: 20px;
                    left: 50%;
                    transform: translateX(-50%);
                    background-color: var(--toolbar-bg);
                    backdrop-filter: blur(10px);
                    color: var(--text-color);
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    height: 44px;
                    padding: 0 16px;
                    border-radius: 22px;
                    box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
                    z-index: 1000;
                    border: 1px solid rgba(255,255,255,0.1);
                    transition: opacity 0.3s;
                }
                #toolbar:hover { opacity: 1; }
                #toolbar button {
                    background: none;
                    border: none;
                    color: var(--text-color);
                    cursor: pointer;
                    width: 32px;
                    height: 32px;
                    border-radius: 50%;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    font-size: 18px;
                    transition: background-color 0.2s;
                }
                #toolbar button:hover { background-color: rgba(255,255,255,0.1); }
                #scale-display { 
                    margin: 0 12px; 
                    width: 40px; 
                    text-align: center; 
                    font-size: 13px;
                    font-weight: 500;
                    font-variant-numeric: tabular-nums;
                }
                #page-container { 
                    flex: 1; 
                    overflow: auto; 
                    padding: 40px 20px; 
                    text-align: center; 
                    box-sizing: border-box;
                    scroll-behavior: smooth;
                }
                canvas { 
                    box-shadow: 0 4px 20px rgba(0,0,0,0.3); 
                    margin-bottom: 24px; 
                    display: inline-block; 
                    vertical-align: top;
                    border-radius: 4px;
                }
                .page-wrapper { position: relative; display: inline-block; margin-bottom: 24px; }
                /* SyncTeX Highlight Overlay */
                .marker {
                    position: absolute;
                    width: 100%;
                    height: 0;
                    border-top: 2px solid rgba(59, 130, 246, 0.8);
                    box-shadow: 0 0 10px rgba(59, 130, 246, 0.5);
                    pointer-events: none;
                    animation: fadeOut 2s forwards;
                    z-index: 100;
                    transform: translateY(-50%);
                }
                @keyframes fadeOut {
                    0% { opacity: 1; }
                    80% { opacity: 1; }
                    100% { opacity: 0; }
                }
                /* Custom Scrollbar */
                ::-webkit-scrollbar { width: 10px; height: 10px; }
                ::-webkit-scrollbar-track { background: transparent; }
                ::-webkit-scrollbar-thumb { background: rgba(255,255,255,0.2); border-radius: 5px; }
                ::-webkit-scrollbar-thumb:hover { background: rgba(255,255,255,0.3); }
            </style>
        </head>
        <body>
            <div id="page-container"></div>
            <!-- Floating Toolbar -->
            <div id="toolbar">
                <button id="zoom-out" title="Zoom Out">
                    <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor"><path d="M14 8a.75.75 0 0 1-.75.75H2.75a.75.75 0 0 1 0-1.5h10.5A.75.75 0 0 1 14 8Z"/></svg>
                </button>
                <span id="scale-display">100%</span>
                <button id="zoom-in" title="Zoom In">
                    <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor"><path d="M8.75 2.75a.75.75 0 0 0-1.5 0v4.5h-4.5a.75.75 0 0 0 0 1.5h4.5v4.5a.75.75 0 0 0 1.5 0v-4.5h4.5a.75.75 0 0 0 0-1.5h-4.5v-4.5Z"/></svg>
                </button>
            </div>
            <script type="module">
                import * as pdfjsLib from '${pdfJsUri}';

                const vscode = acquireVsCodeApi();

                pdfjsLib.GlobalWorkerOptions.workerSrc = '${pdfWorkerUri}';

                const url = '${pdfContentUri}';
                let pdfDoc = null;
                let currentScale = 1.0;
                let pageElements = [];

                // Load PDF
                pdfjsLib.getDocument(url).promise.then(async pdf => {
                    pdfDoc = pdf;
                    console.log('PDF loaded, pages:', pdf.numPages);
                    renderPages();
                });

                document.getElementById('zoom-in').addEventListener('click', () => {
                    currentScale += 0.25;
                    renderPages();
                });

                document.getElementById('zoom-out').addEventListener('click', () => {
                    if (currentScale > 0.5) {
                        currentScale -= 0.25;
                        renderPages();
                    }
                });

                async function renderPages() {
                    const container = document.getElementById('page-container');
                    container.innerHTML = ''; // Clear existing
                    pageElements = [];
                    
                    document.getElementById('scale-display').textContent = Math.round(currentScale * 100) + '%';

                    for (let pageNum = 1; pageNum <= pdfDoc.numPages; pageNum++) {
                        const page = await pdfDoc.getPage(pageNum);
                        const viewport = page.getViewport({ scale: currentScale });

                        // Wrapper for positioning markers
                        const wrapper = document.createElement('div');
                        wrapper.className = 'page-wrapper';
                        
                        const canvas = document.createElement('canvas');
                        const context = canvas.getContext('2d');
                        canvas.height = viewport.height;
                        canvas.width = viewport.width;
                        canvas.id = 'page-' + pageNum;

                        wrapper.appendChild(canvas);
                        container.appendChild(wrapper);

                        pageElements[pageNum] = { 
                            canvas, 
                            viewport, 
                            wrapper // Store wrapper to append marker to
                        };

                        const renderContext = {
                            canvasContext: context,
                            viewport: viewport
                        };
                        
                        // Async render, don't await loop to be faster?
                        // Awaiting ensures order and prevents heavy freeze?
                        await page.render(renderContext).promise;

                        canvas.addEventListener('click', (e) => {
                            if (!e.ctrlKey && !e.metaKey) return;
                            
                            const rect = canvas.getBoundingClientRect();
                            const x = e.clientX - rect.left;
                            const y = e.clientY - rect.top;
                            const pdfPoint = viewport.convertToPdfPoint(x, y);
                            
                            vscode.postMessage({
                                command: 'synctex_inverse',
                                page: pageNum,
                                x: pdfPoint[0],
                                y: pdfPoint[1]
                            });
                        });
                    }
                }

                window.addEventListener('message', event => {
                    const message = event.data;
                    if (message.command === 'synctex_forward') {
                        const { page, x, y } = message;
                        const pageData = pageElements[page];
                        if (pageData) {
                            pageData.canvas.scrollIntoView({ behavior: 'smooth', block: 'center' });
                            
                            // Visual Indicator - Line Highlight
                            const viewPoint = pageData.viewport.convertToViewportPoint(x, y);
                            
                            // Remove existing markers
                            document.querySelectorAll('.marker').forEach(el => el.remove());

                            const marker = document.createElement('div');
                            marker.className = 'marker';
                            // SyncTeX gives us a point, usually baseline. We want a horizontal line there.
                            // viewPoint[1] is Y.
                            marker.style.top = viewPoint[1] + 'px';
                            // We make it span the width of the page wrapper
                            
                            pageData.wrapper.appendChild(marker);

                            // Auto-removed by animation in CSS
                            setTimeout(() => marker.remove(), 2000);
                        }
                    }
                });
            </script>
        </body>
        </html>`;
  }
}
