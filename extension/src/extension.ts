import * as fs from "node:fs";
import * as path from "node:path";
import * as vscode from "vscode";
import { SidecarClient, startSidecar } from "./sidecar";

let sidecar: SidecarClient | undefined;
let handshake: { version?: string; error?: string } = {};

export function activate(context: vscode.ExtensionContext): void {
  try {
    sidecar = startSidecar();
  } catch (err) {
    handshake = { error: err instanceof Error ? err.message : String(err) };
  }

  if (sidecar) {
    sidecar
      .ping()
      .then((result) => {
        handshake = { version: result.version };
      })
      .catch((err: unknown) => {
        handshake = { error: err instanceof Error ? err.message : String(err) };
      });
  }

  context.subscriptions.push(
    vscode.commands.registerCommand("call.openGraph", () => openGraph(context)),
    { dispose: () => sidecar?.dispose() },
  );
}

export function deactivate(): void {
  sidecar?.dispose();
  sidecar = undefined;
}

async function openGraph(context: vscode.ExtensionContext): Promise<void> {
  const panel = vscode.window.createWebviewPanel(
    "call.graph",
    "Call Graph",
    vscode.ViewColumn.One,
    {
      enableScripts: true,
      localResourceRoots: webviewRoots(context),
    },
  );

  const { html, error } = renderWebview(context, panel.webview);
  if (error) {
    handshake = { error };
  }
  panel.webview.html = html;

  const sendHandshake = () => {
    panel.webview.postMessage({
      type: "handshake",
      version: handshake.version,
      error: handshake.error,
    });
  };

  const sub = panel.webview.onDidReceiveMessage((msg: { type?: string }) => {
    if (msg?.type === "ready") {
      sendHandshake();
    }
  });
  panel.onDidDispose(() => sub.dispose());

  if (sidecar && !handshake.version && !handshake.error) {
    try {
      const result = await sidecar.ping();
      handshake = { version: result.version };
    } catch (err) {
      handshake = { error: err instanceof Error ? err.message : String(err) };
    }
  }
  sendHandshake();
}

function webviewRoots(context: vscode.ExtensionContext): vscode.Uri[] {
  return [webviewDir(context)].filter((dir) => fs.existsSync(dir)).map((dir) => vscode.Uri.file(dir));
}

function webviewDir(context: vscode.ExtensionContext): string {
  const packaged = path.join(context.extensionPath, "media");
  if (fs.existsSync(path.join(packaged, "webview.js"))) {
    return packaged;
  }
  return path.join(context.extensionPath, "..", "webview", "dist");
}

function renderWebview(
  context: vscode.ExtensionContext,
  webview: vscode.Webview,
): { html: string; error?: string } {
  const dir = webviewDir(context);
  const jsPath = path.join(dir, "webview.js");
  if (!fs.existsSync(jsPath)) {
    const message =
      "Webview bundle not found. Run `npm run build` in the webview package.";
    return {
      error: message,
      html: `<!DOCTYPE html><html><body><p>${message}</p></body></html>`,
    };
  }
  const jsUri = webview.asWebviewUri(vscode.Uri.file(jsPath));
  const cssPath = path.join(dir, "webview.css");
  const cssTag = fs.existsSync(cssPath)
    ? `<link rel="stylesheet" href="${webview.asWebviewUri(vscode.Uri.file(cssPath))}">`
    : "";
  const nonce = String(Math.random()).slice(2);
  const html = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src ${webview.cspSource} https:; style-src ${webview.cspSource} 'unsafe-inline'; script-src 'nonce-${nonce}';">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  ${cssTag}
</head>
<body>
  <div id="root"></div>
  <script nonce="${nonce}" src="${jsUri}"></script>
</body>
</html>`;
  return { html };
}
