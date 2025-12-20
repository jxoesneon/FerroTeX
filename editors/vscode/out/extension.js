"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.deactivate = exports.activate = void 0;
const vscode_1 = require("vscode");
const node_1 = require("vscode-languageclient/node");
let client;
function activate(context) {
  const config = vscode_1.workspace.getConfiguration("ferrotex");
  const serverPath = config.get("serverPath") || "ferrotexd";
  // If the extension is launched in debug mode then the debug server options are used
  // Otherwise the run options are used
  const serverOptions = {
    run: { command: serverPath },
    debug: { command: serverPath },
  };
  // Options to control the language client
  const clientOptions = {
    // Register the server for plain text documents
    documentSelector: [
      { scheme: "file", language: "latex" },
      { scheme: "file", language: "tex" },
    ],
    synchronize: {
      // Notify the server about file changes to '.clientrc files contained in the workspace
      fileEvents: vscode_1.workspace.createFileSystemWatcher("**/.clientrc"),
    },
  };
  // Create the language client and start the client.
  client = new node_1.LanguageClient(
    "ferrotex",
    "FerroTeX Language Server",
    serverOptions,
    clientOptions,
  );
  // Start the client. This will also launch the server
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
//# sourceMappingURL=extension.js.map
