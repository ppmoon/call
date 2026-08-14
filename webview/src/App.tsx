import type { CSSProperties } from "react";
import {
  Background,
  Controls,
  MiniMap,
  ReactFlow,
  type Edge,
  type Node,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import type { Handshake } from "./handshake";
import { layoutGraph } from "./layout";
import type { Graph, Pin, RunResult, SymbolInfo } from "./types";

export type Ghost = { id: string; prompt: string; sourceId?: string };

export type AppProps = {
  handshake: Handshake;
  graph?: Graph | null;
  pins?: Pin[];
  symbols?: SymbolInfo[];
  ghosts?: Ghost[];
  selectedId?: string | null;
  runOutput?: Record<string, RunResult>;
  highlighted?: string[];
  showExternal?: boolean;
  onToggleExternal?: () => void;
  onSelectEntry?: (entry: string) => void;
  onExpand?: (id: string) => void;
  onCollapse?: (id: string) => void;
  onToggleContainer?: (id: string) => void;
  onUncluster?: (layer: number) => void;
  onPin?: (qualifiedName: string, x: number, y: number) => void;
  onPrompt?: (qualifiedName: string, prompt: string) => void;
  onRunNode?: (qualifiedName: string, argsJson: string) => void;
  onRunProgram?: () => void;
  onScaffold?: (language: string) => void;
  onGhostCreate?: () => void;
  onGhostGenerate?: (ghostId: string) => void;
  onGhostPrompt?: (ghostId: string, prompt: string) => void;
  onConnectGhost?: (sourceId: string, ghostId: string) => void;
  onDragInsert?: (callerId: string, calleeQn: string) => void;
  onPickSymbol?: (qn: string) => void;
  onSelect?: (id: string) => void;
};

function statusText(handshake: Handshake): string {
  switch (handshake.status) {
    case "connecting":
      return "Connecting to Sidecar…";
    case "ok":
      return `Sidecar ${handshake.version}`;
    case "error":
      return handshake.message;
  }
}

export function App(props: AppProps) {
  const graph = props.graph;
  const pins = props.pins ?? [];
  const ghosts = props.ghosts ?? [];
  const highlighted = new Set(props.highlighted ?? []);

  const rfNodes: Node[] = [];
  const rfEdges: Edge[] = [];
  if (graph && !graph.empty) {
    const pos = layoutGraph(graph, pins);
    for (const n of graph.nodes) {
      const p = pos.get(n.id) ?? { x: 0, y: 0 };
      const run = n.qualifiedName ? props.runOutput?.[n.qualifiedName] : undefined;
      rfNodes.push({
        id: n.id,
        position: p,
        data: { label: nodeLabel(n, run, highlighted.has(n.qualifiedName ?? "")) },
        style: nodeStyle(n, highlighted.has(n.qualifiedName ?? "")),
      });
    }
    for (const e of graph.edges) {
      rfEdges.push({
        id: e.id,
        source: e.source,
        target: e.target,
        style: highlighted.has(e.source) && highlighted.has(e.target) ? { stroke: "#eab308" } : undefined,
      });
    }
  }
  for (const g of ghosts) {
    rfNodes.push({
      id: g.id,
      position: { x: 480, y: 40 },
      data: { label: `Ghost: ${g.prompt || "(prompt)"}` },
      style: { border: "1px dashed #888", background: "#222", color: "#ccc", padding: 8 },
    });
    if (g.sourceId) {
      rfEdges.push({ id: `ghost-${g.sourceId}-${g.id}`, source: g.sourceId, target: g.id });
    }
  }

  return (
    <div className="call-shell">
      <div className="call-status" data-testid="sidecar-status">
        {statusText(props.handshake)}
        {graph?.entry ? ` · Entry ${graph.entry}` : ""}
      </div>
      <div className="call-toolbar">
        <label>
          Entry
          <select
            data-testid="entry-select"
            value={graph?.entry ?? ""}
            onChange={(e) => props.onSelectEntry?.(e.target.value)}
          >
            {(graph?.entries ?? []).map((entry) => (
              <option key={entry} value={entry}>
                {entry}
              </option>
            ))}
          </select>
        </label>
        <label>
          <input
            type="checkbox"
            data-testid="show-external"
            checked={props.showExternal ?? graph?.showExternal ?? false}
            onChange={() => props.onToggleExternal?.()}
          />
          show external
        </label>
        <button type="button" onClick={() => props.onRunProgram?.()}>
          Program Run
        </button>
        <button type="button" onClick={() => props.onGhostCreate?.()}>
          Ghost Node
        </button>
        <button type="button" onClick={() => props.onScaffold?.("python")}>
          hello-world
        </button>
      </div>
      <div className="call-body">
        <aside className="call-sidebar" data-testid="symbol-sidebar">
          <div className="call-sidebar-title">Functions</div>
          {(props.symbols ?? []).map((sym) => (
            <button
              type="button"
              key={sym.qualifiedName}
              draggable
              onDragStart={(e) => e.dataTransfer.setData("text/qn", sym.qualifiedName)}
              onClick={() => props.onPickSymbol?.(sym.qualifiedName)}
            >
              {sym.qualifiedName}
            </button>
          ))}
        </aside>
        <div
          className="call-canvas"
          data-testid="graph-canvas"
          onDragOver={(e) => e.preventDefault()}
          onDrop={(e) => {
            const qn = e.dataTransfer.getData("text/qn");
            const caller = props.selectedId;
            if (qn && caller) {
              props.onDragInsert?.(caller, qn);
            }
          }}
        >
          {graph?.empty ? (
            <div data-testid="empty-state" className="call-empty">
              No Entry found.
              <button type="button" onClick={() => props.onScaffold?.("python")}>
                Generate hello-world (Python)
              </button>
            </div>
          ) : (
            <ReactFlow
              nodes={rfNodes}
              edges={rfEdges}
              onNodeClick={(_, node) => {
                props.onSelect?.(node.id);
                const gnode = graph?.nodes.find((n) => n.id === node.id);
                if (gnode?.kind === "cluster") {
                  props.onUncluster?.(gnode.layer);
                } else if (gnode?.kind === "container") {
                  props.onToggleContainer?.(gnode.id);
                } else if (gnode?.expandable) {
                  props.onExpand?.(gnode.id);
                }
              }}
              onNodeDoubleClick={(_, node) => props.onCollapse?.(node.id)}
              onNodeDragStop={(_, node) => {
                const gnode = graph?.nodes.find((n) => n.id === node.id);
                if (gnode?.qualifiedName) {
                  props.onPin?.(gnode.qualifiedName, node.position.x, node.position.y);
                }
              }}
              proOptions={{ hideAttribution: true }}
              fitView
              onConnect={(c) => {
                if (c.target?.startsWith("ghost-") && c.source) {
                  props.onConnectGhost?.(c.source, c.target);
                }
              }}
            >
              <Background />
              <Controls />
              <MiniMap />
            </ReactFlow>
          )}
        </div>
        <Inspector
          graph={graph}
          selectedId={props.selectedId}
          ghosts={ghosts}
          runOutput={props.runOutput}
          onPrompt={props.onPrompt}
          onRunNode={props.onRunNode}
          onGhostPrompt={props.onGhostPrompt}
          onGhostGenerate={props.onGhostGenerate}
          onConnectGhost={props.onConnectGhost}
        />
      </div>
    </div>
  );
}

function Inspector(props: {
  graph?: Graph | null;
  selectedId?: string | null;
  ghosts: Ghost[];
  runOutput?: Record<string, RunResult>;
  onPrompt?: (qualifiedName: string, prompt: string) => void;
  onRunNode?: (qualifiedName: string, argsJson: string) => void;
  onGhostPrompt?: (ghostId: string, prompt: string) => void;
  onGhostGenerate?: (ghostId: string) => void;
  onConnectGhost?: (sourceId: string, ghostId: string) => void;
}) {
  const ghost = props.ghosts.find((g) => g.id === props.selectedId);
  const node = props.graph?.nodes.find((n) => n.id === props.selectedId);
  if (ghost) {
    return (
      <aside className="call-inspector" data-testid="inspector">
        <h2>Ghost Node</h2>
        <textarea
          data-testid="ghost-prompt"
          value={ghost.prompt}
          onChange={(e) => props.onGhostPrompt?.(ghost.id, e.target.value)}
        />
        <button type="button" onClick={() => props.onGhostGenerate?.(ghost.id)}>
          Generate
        </button>
      </aside>
    );
  }
  if (!node?.qualifiedName) {
    return (
      <aside className="call-inspector" data-testid="inspector">
        Select a Function Node
      </aside>
    );
  }
  const run = props.runOutput?.[node.qualifiedName];
  return (
    <aside className="call-inspector" data-testid="inspector">
      <h2>{node.qualifiedName}</h2>
      <p>{node.signature}</p>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          const form = e.target as HTMLFormElement;
          const prompt = (form.elements.namedItem("prompt") as HTMLTextAreaElement).value;
          props.onPrompt?.(node.qualifiedName!, prompt);
        }}
      >
        <textarea name="prompt" data-testid="node-prompt" placeholder="Prompt the AI…" />
        <button type="submit">Propose edit</button>
      </form>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          const form = e.target as HTMLFormElement;
          const args = (form.elements.namedItem("args") as HTMLTextAreaElement).value || "{}";
          props.onRunNode?.(node.qualifiedName!, args);
        }}
      >
        <textarea name="args" data-testid="node-args" defaultValue="{}" />
        <button type="submit">Node Run</button>
      </form>
      {run ? (
        <pre data-testid="node-output">
          {run.ok ? run.stdout : run.stderr || "failed"}
        </pre>
      ) : null}
    </aside>
  );
}

function nodeLabel(
  n: { label: string; kind: string; expandable: boolean },
  run: RunResult | undefined,
  lit: boolean,
): string {
  const bits = [n.label];
  if (n.kind === "cluster") {
    bits.push("(cluster)");
  }
  if (n.expandable && n.kind === "function") {
    bits.push("+");
  }
  if (lit) {
    bits.push("●");
  }
  if (run) {
    bits.push(run.ok ? "ok" : "err");
  }
  return bits.join(" ");
}

function nodeStyle(
  n: { kind: string; external: boolean },
  lit: boolean,
): CSSProperties {
  const bg = lit ? "#854d0e" : n.kind === "container" ? "#1f2937" : n.external ? "#3f3f46" : "#111827";
  return {
    padding: 8,
    borderRadius: 6,
    border: n.kind === "cluster" ? "1px dashed #f59e0b" : "1px solid #4b5563",
    background: bg,
    color: "#e5e7eb",
    fontSize: 12,
    minWidth: 140,
  };
}
