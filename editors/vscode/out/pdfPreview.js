"use strict";
var __createBinding =
  (this && this.__createBinding) ||
  (Object.create
    ? function (o, m, k, k2) {
        if (k2 === undefined) k2 = k;
        var desc = Object.getOwnPropertyDescriptor(m, k);
        if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
          desc = {
            enumerable: true,
            get: function () {
              return m[k];
            },
          };
        }
        Object.defineProperty(o, k2, desc);
      }
    : function (o, m, k, k2) {
        if (k2 === undefined) k2 = k;
        o[k2] = m[k];
      });
var __setModuleDefault =
  (this && this.__setModuleDefault) ||
  (Object.create
    ? function (o, v) {
        Object.defineProperty(o, "default", { enumerable: true, value: v });
      }
    : function (o, v) {
        o["default"] = v;
      });
var __importStar =
  (this && this.__importStar) ||
  function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null)
      for (var k in mod)
        if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k))
          __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
  };
Object.defineProperty(exports, "__esModule", { value: true });
exports.PdfPreviewProvider = void 0;
const vscode = __importStar(require("vscode"));
/**
 * integrated PDF Viewer provider for FerroTeX.
 *
 * Uses `pdfjs-dist` to render PDFs in a Webview.
 * Supports SyncTeX (Forward and Inverse).
 */
class PdfPreviewProvider {
  constructor(extensionUri, client) {
    this.extensionUri = extensionUri;
    this.client = client;
    this.panels = new Map();
  }
  openCustomDocument(uri, _openContext, _token) {
    return { uri, dispose: () => {} };
  }
  async resolveCustomEditor(document, webviewPanel, _token) {
    this.panels.set(document.uri.toString(), webviewPanel);
    webviewPanel.onDidDispose(() => {
      this.panels.delete(document.uri.toString());
    });
    webviewPanel.webview.options = {
      enableScripts: true,
      localResourceRoots: [
        vscode.Uri.joinPath(this.extensionUri, "node_modules"),
        vscode.Uri.file(document.uri.path).with({
          path: document.uri.path.substring(0, document.uri.path.lastIndexOf("/") + 1),
        }),
      ],
    };
    webviewPanel.webview.html = this.getHtmlForWebview(webviewPanel.webview, document.uri);
    webviewPanel.webview.onDidReceiveMessage(async (e) => {
      if (e.command === "synctex_inverse") {
        const args = [document.uri.toString(), e.page, e.x, e.y];
        const result = await this.client.sendRequest("workspace/executeCommand", {
          command: "ferrotex.synctex_inverse",
          arguments: args,
        });
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
  reveal(pdfUri, page, x, y) {
    const panel = this.panels.get(pdfUri.toString());
    if (panel) {
      panel.reveal();
      panel.webview.postMessage({ command: "synctex_forward", page, x, y });
    }
  }
  getHtmlForWebview(webview, pdfUri) {
    const pdfJsUri = webview.asWebviewUri(
      vscode.Uri.joinPath(this.extensionUri, "node_modules", "pdfjs-dist", "build", "pdf.mjs"),
    );
    const pdfWorkerUri = webview.asWebviewUri(
      vscode.Uri.joinPath(
        this.extensionUri,
        "node_modules",
        "pdfjs-dist",
        "build",
        "pdf.worker.mjs",
      ),
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
                #toolbar {
                    background-color: #323639;
                    color: white;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    height: 40px;
                    width: 100%;
                    box-shadow: 0 1px 3px rgba(0,0,0,0.2);
                    z-index: 1000;
                    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
                }
                #toolbar button {
                    background: none;
                    border: none;
                    color: white;
                    cursor: pointer;
                    padding: 5px 10px;
                    font-size: 14px;
                }
                #toolbar button:hover { background-color: rgba(255,255,255,0.1); border-radius: 4px; }
                #page-container { 
                    flex: 1; 
                    overflow: auto; 
                    padding: 20px; 
                    text-align: center; 
                    box-sizing: border-box;
                }
                canvas { box-shadow: 0 0 10px rgba(0,0,0,0.5); margin-bottom: 20px; display: inline-block; vertical-align: top; }
                .page-wrapper { margin-bottom: 20px; position: relative; display: inline-block; }
                /* SyncTeX Highlight Overlay */
                .marker {
                    position: absolute;
                    width: 20px;
                    height: 20px;
                    background-color: rgba(255, 0, 0, 0.4);
                    border: 2px solid red;
                    border-radius: 50%;
                    transform: translate(-50%, -50%);
                    pointer-events: none;
                    transition: opacity 0.5s;
                    z-index: 100;
                }
            </style>
        </head>
        <body>
            <div id="toolbar">
                <button id="zoom-out">-</button>
                <span id="scale-display" style="margin: 0 10px; width: 50px; text-align: center;">100%</span>
                <button id="zoom-in">+</button>
            </div>
            <div id="page-container"></div>
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
                            
                            // Visual Indicator
                            const viewPoint = pageData.viewport.convertToViewportPoint(x, y);
                            
                            // Remove existing markers
                            document.querySelectorAll('.marker').forEach(el => el.remove());

                            const marker = document.createElement('div');
                            marker.className = 'marker';
                            marker.style.left = viewPoint[0] + 'px';
                            marker.style.top = viewPoint[1] + 'px';
                            
                            pageData.wrapper.appendChild(marker);

                            // Auto-fade
                            setTimeout(() => {
                                marker.style.opacity = '0';
                                setTimeout(() => marker.remove(), 500);
                            }, 3000);
                        }
                    }
                });
            </script>
        </body>
        </html>`;
  }
}
exports.PdfPreviewProvider = PdfPreviewProvider;
PdfPreviewProvider.viewType = "ferrotex.pdfPreview";
//# sourceMappingURL=pdfPreview.js.map
