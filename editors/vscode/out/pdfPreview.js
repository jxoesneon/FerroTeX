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
exports.PdfPreviewProvider = void 0;
const vscode = __importStar(require("vscode"));
/**
 * Integrated PDF Viewer provider for FerroTeX.
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
        return { uri, dispose: () => { } };
    }
    async resolveCustomEditor(document, webviewPanel, _token) {
        this.panels.set(document.uri.toString(), webviewPanel);
        webviewPanel.onDidDispose(() => {
            this.panels.delete(document.uri.toString());
        });
        const docDir = vscode.Uri.joinPath(document.uri, "..");
        webviewPanel.webview.options = {
            enableScripts: true,
            localResourceRoots: [
                vscode.Uri.joinPath(this.extensionUri, "dist"),
                vscode.Uri.joinPath(this.extensionUri, "dist", "pdfjs"),
                docDir,
                vscode.Uri.file(document.uri.path).with({
                    path: document.uri.path.substring(0, document.uri.path.lastIndexOf("/") + 1),
                }),
            ],
        };
        webviewPanel.webview.html = this.getHtmlForWebview(webviewPanel.webview, document.uri);
        // Watch for PDF recompilations and auto-reload preview
        const watcher = vscode.workspace.createFileSystemWatcher(new vscode.RelativePattern(docDir, "*.pdf"));
        watcher.onDidChange((uri) => {
            if (uri.path === document.uri.path) {
                webviewPanel.webview.postMessage({ command: "reload_pdf" });
            }
        });
        webviewPanel.onDidDispose(() => {
            watcher.dispose();
        });
        webviewPanel.webview.onDidReceiveMessage(async (e) => {
            if (e.command === "synctex_inverse") {
                const args = [document.uri.toString(), e.page, e.x, e.y];
                const result = (await this.client.sendRequest("workspace/executeCommand", {
                    command: "ferrotex.synctex_inverse",
                    arguments: args,
                }));
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
        const pdfJsUri = webview.asWebviewUri(vscode.Uri.joinPath(this.extensionUri, "dist", "pdfjs", "pdf.mjs"));
        const pdfWorkerUri = webview.asWebviewUri(vscode.Uri.joinPath(this.extensionUri, "dist", "pdfjs", "pdf.worker.mjs"));
        const pdfContentUri = webview.asWebviewUri(pdfUri);
        return `<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>PDF Preview</title>
            <style>
                :root {
                    --bg-color: #1e1e1e;
                    --toolbar-bg: rgba(30, 30, 30, 0.85);
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
                    bottom: 24px;
                    left: 50%;
                    transform: translateX(-50%);
                    background-color: var(--toolbar-bg);
                    backdrop-filter: blur(12px);
                    color: var(--text-color);
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    height: 40px;
                    padding: 0 14px;
                    border-radius: 20px;
                    box-shadow: 0 6px 16px rgba(0, 0, 0, 0.4), 0 0 0 1px rgba(255, 255, 255, 0.1);
                    z-index: 1000;
                    user-select: none;
                }
                #toolbar button {
                    background: none;
                    border: none;
                    color: var(--text-color);
                    cursor: pointer;
                    width: 30px;
                    height: 30px;
                    border-radius: 50%;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    font-size: 16px;
                    transition: background-color 0.2s;
                }
                #toolbar button:hover { background-color: rgba(255,255,255,0.15); }
                #scale-display { 
                    margin: 0 12px; 
                    min-width: 44px; 
                    text-align: center; 
                    font-size: 13px;
                    font-weight: 500;
                    font-variant-numeric: tabular-nums;
                }
                #page-container { 
                    flex: 1; 
                    overflow-y: auto; 
                    overflow-x: auto;
                    padding: 24px 16px 80px 16px; 
                    display: flex;
                    flex-direction: column;
                    align-items: center;
                    box-sizing: border-box;
                    scroll-behavior: smooth;
                }
                .page-wrapper { 
                    position: relative; 
                    display: block; 
                    margin: 0 auto 24px auto; 
                    background-color: #ffffff;
                    border-radius: 3px;
                    box-shadow: 0 6px 20px rgba(0, 0, 0, 0.45);
                    overflow: hidden;
                    flex-shrink: 0;
                }
                canvas { 
                    display: block; 
                }
                /* SyncTeX Highlight Overlay */
                .marker {
                    position: absolute;
                    width: 100%;
                    height: 0;
                    border-top: 2px solid rgba(59, 130, 246, 0.85);
                    box-shadow: 0 0 10px rgba(59, 130, 246, 0.6);
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
                .error-container {
                    margin: 40px auto;
                    padding: 20px 24px;
                    background: rgba(239, 68, 68, 0.15);
                    border: 1px solid rgba(239, 68, 68, 0.3);
                    border-radius: 8px;
                    max-width: 450px;
                    text-align: center;
                    color: #fca5a5;
                }
                /* Custom Scrollbar */
                ::-webkit-scrollbar { width: 8px; height: 8px; }
                ::-webkit-scrollbar-track { background: transparent; }
                ::-webkit-scrollbar-thumb { background: rgba(255,255,255,0.2); border-radius: 4px; }
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

                let pdfDoc = null;
                let currentScale = 1.0;
                let pageElements = [];
                let workerBlobUrl = null;

                async function ensureWorker() {
                    if (workerBlobUrl) return;
                    try {
                        const resp = await fetch('${pdfWorkerUri}');
                        const code = await resp.text();
                        const blob = new Blob([code], { type: 'application/javascript' });
                        workerBlobUrl = URL.createObjectURL(blob);
                        pdfjsLib.GlobalWorkerOptions.workerSrc = workerBlobUrl;
                    } catch (e) {
                        console.warn('Worker fallback:', e);
                        pdfjsLib.GlobalWorkerOptions.workerSrc = '';
                    }
                }

                async function loadPdf() {
                    try {
                        await ensureWorker();
                        const resp = await fetch('${pdfContentUri}');
                        if (!resp.ok) {
                            throw new Error('Failed to load PDF file (' + resp.status + ')');
                        }
                        const buffer = await resp.arrayBuffer();
                        const loadingTask = pdfjsLib.getDocument({ data: new Uint8Array(buffer) });
                        pdfDoc = await loadingTask.promise;

                        const container = document.getElementById('page-container');
                        if (pdfDoc.numPages > 0 && container.clientWidth > 0) {
                            const firstPage = await pdfDoc.getPage(1);
                            const baseViewport = firstPage.getViewport({ scale: 1.0 });
                            const fitScale = (container.clientWidth - 48) / baseViewport.width;
                            currentScale = Math.max(0.4, Math.min(1.0, Math.round(fitScale * 20) / 20));
                        }

                        await renderPages();
                    } catch (err) {
                        console.error('PDF preview error:', err);
                        const container = document.getElementById('page-container');
                        container.innerHTML = '<div class="error-container"><h3>Unable to load PDF Preview</h3><p>' + (err.message || err) + '</p></div>';
                    }
                }

                async function renderPages() {
                    if (!pdfDoc) return;
                    const container = document.getElementById('page-container');
                    container.innerHTML = '';
                    pageElements = [];
                    
                    document.getElementById('scale-display').textContent = Math.round(currentScale * 100) + '%';

                    for (let pageNum = 1; pageNum <= pdfDoc.numPages; pageNum++) {
                        const page = await pdfDoc.getPage(pageNum);
                        const viewport = page.getViewport({ scale: currentScale });

                        const wrapper = document.createElement('div');
                        wrapper.className = 'page-wrapper';
                        wrapper.style.width = viewport.width + 'px';
                        wrapper.style.height = viewport.height + 'px';
                        
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
                            wrapper
                        };

                        const renderContext = {
                            canvasContext: context,
                            viewport: viewport
                        };
                        
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

                document.getElementById('zoom-in').addEventListener('click', () => {
                    currentScale += 0.20;
                    renderPages();
                });

                document.getElementById('zoom-out').addEventListener('click', () => {
                    if (currentScale > 0.3) {
                        currentScale -= 0.20;
                        renderPages();
                    }
                });

                window.addEventListener('message', event => {
                    const message = event.data;
                    if (message.command === 'reload_pdf') {
                        loadPdf();
                    } else if (message.command === 'synctex_forward') {
                        const { page, x, y } = message;
                        const pageData = pageElements[page];
                        if (pageData) {
                            pageData.canvas.scrollIntoView({ behavior: 'smooth', block: 'center' });
                            
                            const viewPoint = pageData.viewport.convertToViewportPoint(x, y);
                            
                            document.querySelectorAll('.marker').forEach(el => el.remove());

                            const marker = document.createElement('div');
                            marker.className = 'marker';
                            marker.style.top = viewPoint[1] + 'px';
                            
                            pageData.wrapper.appendChild(marker);
                            setTimeout(() => marker.remove(), 2000);
                        }
                    }
                });

                // Initial PDF loading
                loadPdf();
            </script>
        </body>
        </html>`;
    }
}
exports.PdfPreviewProvider = PdfPreviewProvider;
PdfPreviewProvider.viewType = "ferrotex.pdfPreview";
//# sourceMappingURL=pdfPreview.js.map