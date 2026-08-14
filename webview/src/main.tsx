import { useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import { App, type Ghost } from "./App";
import type { Handshake, HandshakeMessage } from "./handshake";
import type { Graph, Pin, RunResult, SymbolInfo } from "./types";
import "./main.css";

declare function acquireVsCodeApi(): {
  postMessage(message: unknown): void;
};

const vscode = acquireVsCodeApi();

type HostState = {
  handshake: Handshake;
  graph: Graph | null;
  pins: Pin[];
  symbols: SymbolInfo[];
  ghosts: Ghost[];
  selectedId: string | null;
  runOutput: Record<string, RunResult>;
  highlighted: string[];
  showExternal: boolean;
};

function Host() {
  const [state, setState] = useState<HostState>({
    handshake: { status: "connecting" },
    graph: null,
    pins: [],
    symbols: [],
    ghosts: [],
    selectedId: null,
    runOutput: {},
    highlighted: [],
    showExternal: false,
  });

  useEffect(() => {
    const onMessage = (event: MessageEvent) => {
      const data = event.data;
      if (!data?.type) {
        return;
      }
      if (data.type === "handshake") {
        const msg = data as HandshakeMessage;
        if (msg.error) {
          setState((s) => ({ ...s, handshake: { status: "error", message: msg.error! } }));
        } else if (msg.version) {
          setState((s) => ({ ...s, handshake: { status: "ok", version: msg.version! } }));
        }
        return;
      }
      if (data.type === "graph") {
        setState((s) => ({
          ...s,
          graph: data.graph,
          pins: data.pins ?? s.pins,
          symbols: data.symbols ?? s.symbols,
          highlighted: data.highlighted ?? s.highlighted,
          showExternal: data.graph?.showExternal ?? s.showExternal,
        }));
        return;
      }
      if (data.type === "run") {
        setState((s) => ({
          ...s,
          runOutput: { ...s.runOutput, [data.qualifiedName]: data.result },
          highlighted: data.hits ?? s.highlighted,
        }));
      }
    };
    window.addEventListener("message", onMessage);
    vscode.postMessage({ type: "ready" });
    return () => window.removeEventListener("message", onMessage);
  }, []);

  const send = (message: unknown) => vscode.postMessage(message);

  return (
    <App
      handshake={state.handshake}
      graph={state.graph}
      pins={state.pins}
      symbols={state.symbols}
      ghosts={state.ghosts}
      selectedId={state.selectedId}
      runOutput={state.runOutput}
      highlighted={state.highlighted}
      showExternal={state.showExternal}
      onToggleExternal={() => send({ type: "toggleExternal" })}
      onSelectEntry={(entry) => send({ type: "selectEntry", entry })}
      onExpand={(id) => send({ type: "expand", id })}
      onCollapse={(id) => send({ type: "collapse", id })}
      onToggleContainer={(id) => send({ type: "toggleContainer", id })}
      onUncluster={(layer) => send({ type: "uncluster", layer })}
      onPin={(qualifiedName, x, y) => send({ type: "pin", qualifiedName, x, y })}
      onPrompt={(qualifiedName, prompt) => send({ type: "prompt", qualifiedName, prompt })}
      onRunNode={(qualifiedName, argsJson) => send({ type: "runNode", qualifiedName, argsJson })}
      onRunProgram={() => send({ type: "runProgram" })}
      onScaffold={(language) => send({ type: "scaffold", language })}
      onGhostCreate={() =>
        setState((s) => ({
          ...s,
          ghosts: [...s.ghosts, { id: `ghost-${s.ghosts.length + 1}`, prompt: "" }],
          selectedId: `ghost-${s.ghosts.length + 1}`,
        }))
      }
      onGhostPrompt={(id, prompt) =>
        setState((s) => ({
          ...s,
          ghosts: s.ghosts.map((g) => (g.id === id ? { ...g, prompt } : g)),
        }))
      }
      onGhostGenerate={(ghostId) => {
        const ghost = state.ghosts.find((g) => g.id === ghostId);
        send({ type: "ghostGenerate", ghost });
      }}
      onConnectGhost={(sourceId, ghostId) =>
        setState((s) => ({
          ...s,
          ghosts: s.ghosts.map((g) => (g.id === ghostId ? { ...g, sourceId } : g)),
        }))
      }
      onDragInsert={(callerId, calleeQn) => send({ type: "dragInsert", callerId, calleeQn })}
      onPickSymbol={(qn) => setState((s) => ({ ...s, selectedId: qn }))}
      onSelect={(id) => setState((s) => ({ ...s, selectedId: id }))}
    />
  );
}

const root = document.getElementById("root");
if (root) {
  createRoot(root).render(<Host />);
}
