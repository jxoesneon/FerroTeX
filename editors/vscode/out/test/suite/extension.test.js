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
const assert = __importStar(require("assert"));
const vscode = __importStar(require("vscode"));
suite("Extension Test Suite", () => {
  vscode.window.showInformationMessage("Start all tests.");
  test("Extension should be present", () => {
    assert.ok(vscode.extensions.getExtension("ferrotex.ferrotex"));
  });
  test("Extension should activate", async () => {
    const ext = vscode.extensions.getExtension("ferrotex.ferrotex");
    if (ext) {
      await ext.activate();
      assert.strictEqual(ext.isActive, true);
    }
  });
  test("Commands should be registered", async () => {
    const commands = await vscode.commands.getCommands(true);
    assert.ok(commands.includes("ferrotex.build"));
    assert.ok(commands.includes("ferrotex.syncToPdf"));
  });
  test("Configuration should have default serverPath", () => {
    const config = vscode.workspace.getConfiguration("ferrotex");
    const serverPath = config.get("serverPath");
    assert.strictEqual(serverPath, "ferrotexd");
  });
});
//# sourceMappingURL=extension.test.js.map
