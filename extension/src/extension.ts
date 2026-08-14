import * as fs from "node:fs";
import * as path from "node:path";
import * as vscode from "vscode";
import { DiffGate, ghostFunctionName, proposeAndMaybeApply } from "./edit";
import { httpLlmComplete, LlmComplete, mockLlmComplete } from "./llm";
import { lspCallees } from "./lsp";
import { SidecarClient, startSidecar } from "./sidecar";

type Graph = {
  entry?: string;
  nodes: {
    id: string;
    kind: string;
    qualifiedName?: string;
    file?: string;
    line?: number;
    signature?: string;
    layer: number;
  }[];
  edges: { id: string; source: string; target: string }[];
  empty: boolean;
  entries: string[];
  showExternal: boolean;
};

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
      .then((r) => {
        handshake = { version: r.version };
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
  const folder = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  const panel = vscode.window.createWebviewPanel("call.graph", "Call Graph", vscode.ViewColumn.One, {
    enableScripts: true,
    localResourceRoots: webviewRoots(context),
  });
  const rendered = renderWebview(context, panel.webview);
  if (rendered.error) {
    handshake = { error: rendered.error };
  }
  panel.webview.html = rendered.html;
  const session = new Session(panel, folder);
  const sub = panel.webview.onDidReceiveMessage((msg) => session.handle(msg));
  panel.onDidDispose(() => sub.dispose());
  await session.boot();
}

class Session {
  expanded: string[] = [];
  collapsedContainers: string[] = [];
  unclusteredLayers: number[] = [];
  showExternal = false;
  entry: string | undefined;
  graph: Graph | null = null;

  constructor(
    private readonly panel: vscode.WebviewPanel,
    private readonly folder: string | undefined,
  ) {}

  async boot(): Promise<void> {
    if (sidecar && !handshake.version && !handshake.error) {
      try {
        handshake = { version: (await sidecar.ping()).version };
      } catch (err) {
        handshake = { error: err instanceof Error ? err.message : String(err) };
      }
    }
    this.panel.webview.postMessage({
      type: "handshake",
      version: handshake.version,
      error: handshake.error,
    });
    if (this.folder && sidecar) {
      await this.refresh();
    }
  }

  async handle(msg: { type?: string; [k: string]: unknown }): Promise<void> {
    if (!msg?.type) {
      return;
    }
    try {
      switch (msg.type) {
        case "ready":
          await this.boot();
          break;
        case "toggleExternal":
          this.showExternal = !this.showExternal;
          await this.refresh();
          break;
        case "selectEntry":
          this.entry = String(msg.entry);
          this.expanded = [];
          await this.refresh();
          break;
        case "expand":
          this.expanded.push(String(msg.id));
          await this.refresh();
          break;
        case "collapse":
          this.expanded = this.expanded.filter((id) => id !== msg.id);
          await this.refresh();
          break;
        case "toggleContainer": {
          const id = String(msg.id);
          this.collapsedContainers = this.collapsedContainers.includes(id)
            ? this.collapsedContainers.filter((x) => x !== id)
            : [...this.collapsedContainers, id];
          await this.refresh();
          break;
        }
        case "uncluster":
          this.unclusteredLayers.push(Number(msg.layer));
          await this.refresh();
          break;
        case "pin":
          if (this.folder) {
            await sidecar?.rpc("meta.pin", {
              workspaceRoot: this.folder,
              qualifiedName: msg.qualifiedName,
              x: msg.x,
              y: msg.y,
            });
            await this.refresh();
          }
          break;
        case "prompt":
          await this.edit(String(msg.qualifiedName), String(msg.prompt), "edit");
          break;
        case "runNode":
          await this.runNode(String(msg.qualifiedName), String(msg.argsJson ?? "{}"));
          break;
        case "runProgram":
          await this.runProgram();
          break;
        case "scaffold":
          await this.scaffold(String(msg.language ?? "python"));
          break;
        case "ghostGenerate":
          await this.ghostGenerate(msg.ghost as { id: string; prompt: string; sourceId?: string });
          break;
        case "dragInsert":
          await this.dragInsert(String(msg.callerId), String(msg.calleeQn));
          break;
        default:
          break;
      }
    } catch (err) {
      void vscode.window.showErrorMessage(err instanceof Error ? err.message : String(err));
    }
  }

  async refresh(): Promise<void> {
    if (!this.folder || !sidecar) {
      return;
    }
    const graph = await sidecar.rpc<Graph>("graph.build", {
      workspaceRoot: this.folder,
      entry: this.entry,
      depth: 2,
      showExternal: this.showExternal,
      expanded: this.expanded,
      collapsedContainers: this.collapsedContainers,
      unclusteredLayers: this.unclusteredLayers,
    });
    this.graph = graph;
    this.entry = graph.entry;
    const meta = await sidecar.rpc<{ pins: { qualifiedName: string; x: number; y: number }[] }>("meta.load", {
      workspaceRoot: this.folder,
    });
    const symbols = await sidecar.rpc("graph.symbols", { workspaceRoot: this.folder });
    this.panel.webview.postMessage({ type: "graph", graph, pins: meta.pins ?? [], symbols });
    void Promise.all(
      graph.nodes
        .filter((n) => n.kind === "function" && n.file && n.qualifiedName)
        .slice(0, 12)
        .map((n) => lspCallees(n.file!, n.line ?? 1)),
    );
  }

  pythonPath(): string {
    const cfg = vscode.workspace.getConfiguration("python");
    return cfg.get<string>("defaultInterpreterPath") || cfg.get<string>("pythonPath") || "python3";
  }

  async llm(): Promise<LlmComplete> {
    const cfg = vscode.workspace.getConfiguration("call");
    if (cfg.get<boolean>("useMockLlm")) {
      return mockLlmComplete();
    }
    const apiKey = cfg.get<string>("openaiApiKey") ?? "";
    if (!apiKey) {
      throw new Error("Set call.openaiApiKey (BYO key) before editing. No code was sent anywhere.");
    }
    return httpLlmComplete({
      apiKey,
      baseUrl: cfg.get<string>("openaiBaseUrl") || "https://api.openai.com/v1",
      model: cfg.get<string>("openaiModel") || "gpt-4.1-mini",
    });
  }

  async gate(): Promise<DiffGate> {
    if (vscode.workspace.getConfiguration("call").get<boolean>("useMockLlm")) {
      return async () => "accept";
    }
    return async ({ originalPath, proposedPath }) => {
      await vscode.commands.executeCommand(
        "vscode.diff",
        vscode.Uri.file(originalPath),
        vscode.Uri.file(proposedPath),
        "Call Diff Gate",
      );
      const pick = await vscode.window.showInformationMessage("Apply this edit?", "Accept", "Reject");
      return pick === "Accept" ? "accept" : "reject";
    };
  }

  async edit(qn: string, prompt: string, instruction: string): Promise<void> {
    if (!this.folder || !sidecar) {
      return;
    }
    const extracted = await sidecar.rpc<{
      file: string;
      source: string;
      signature: string;
      neighbors: { signature: string }[];
    }>("edit.extract", { workspaceRoot: this.folder, qualifiedName: qn });
    const complete = await this.llm();
    const result = await proposeAndMaybeApply({
      workspaceRoot: this.folder,
      filePath: extracted.file,
      signature: extracted.signature,
      source: extracted.source,
      prompt,
      neighbors: extracted.neighbors.map((n) => n.signature).join(", "),
      instruction,
      complete,
      gate: await this.gate(),
      appendPrompt: (qualifiedName, text) => {
        void sidecar?.rpc("meta.appendPrompt", {
          workspaceRoot: this.folder,
          qualifiedName,
          prompt: text,
        });
      },
      qualifiedName: qn,
    });
    if (result.applied) {
      await sidecar.rpc("meta.snapshot", { workspaceRoot: this.folder });
      await this.refresh();
    }
  }

  async ghostGenerate(ghost: { id: string; prompt: string; sourceId?: string }): Promise<void> {
    if (!ghost.sourceId) {
      throw new Error("Connect the Ghost Node with a Call Edge before generating.");
    }
    if (ghost.sourceId.startsWith("cluster:") || ghost.sourceId.startsWith("ext:")) {
      throw new Error("Cannot generate from a Cluster or external node.");
    }
    await this.edit(ghost.sourceId, ghost.prompt, `ghost:${ghostFunctionName(ghost.prompt)}`);
  }

  async dragInsert(callerId: string, calleeQn: string): Promise<void> {
    if (callerId.startsWith("cluster:") || callerId.startsWith("ext:")) {
      throw new Error("Cannot Drag-Insert onto a Cluster or external node.");
    }
    if (calleeQn.startsWith("cluster:") || calleeQn.startsWith("ext:")) {
      throw new Error("Cannot Drag-Insert a Cluster or external function.");
    }
    await this.edit(callerId, `insert call to ${calleeQn}`, `insert-call:${calleeQn}`);
  }

  async runNode(qn: string, argsJson: string): Promise<void> {
    if (!this.folder || !sidecar) {
      return;
    }
    JSON.parse(argsJson);
    const result = await sidecar.rpc("run.node", {
      workspaceRoot: this.folder,
      qualifiedName: qn,
      argsJson,
      pythonPath: this.pythonPath(),
    });
    await sidecar.rpc("meta.runArgs", {
      workspaceRoot: this.folder,
      qualifiedName: qn,
      argsJson,
    });
    this.panel.webview.postMessage({ type: "run", qualifiedName: qn, result });
  }

  async runProgram(): Promise<void> {
    if (!this.folder || !sidecar || !this.entry) {
      return;
    }
    const result = await sidecar.rpc<{ hits?: string[]; failedAt?: string; ok: boolean; stderr: string }>(
      "run.program",
      { workspaceRoot: this.folder, entry: this.entry, pythonPath: this.pythonPath() },
    );
    this.panel.webview.postMessage({
      type: "run",
      qualifiedName: this.entry,
      result,
      hits: result.hits ?? [],
    });
  }

  async scaffold(language: string): Promise<void> {
    if (!this.folder || !sidecar) {
      return;
    }
    await sidecar.rpc("scaffold.helloWorld", { workspaceRoot: this.folder, language });
    await this.refresh();
  }
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
    const message = "Webview bundle not found. Run `npm run build` in the webview package.";
    return { error: message, html: `<!DOCTYPE html><html><body><p>${message}</p></body></html>` };
  }
  const jsUri = webview.asWebviewUri(vscode.Uri.file(jsPath));
  const cssPath = path.join(dir, "webview.css");
  const cssTag = fs.existsSync(cssPath)
    ? `<link rel="stylesheet" href="${webview.asWebviewUri(vscode.Uri.file(cssPath))}">`
    : "";
  const nonce = String(Math.random()).slice(2);
  return {
    html: `<!DOCTYPE html><html><head>
<meta charset="UTF-8">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src ${webview.cspSource} https:; style-src ${webview.cspSource} 'unsafe-inline'; script-src 'nonce-${nonce}';">
${cssTag}
</head><body><div id="root"></div>
<script nonce="${nonce}" src="${jsUri}"></script>
</body></html>`,
  };
}
