import { ReactFlow } from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import type { Handshake } from "./handshake";

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

export function App({ handshake }: { handshake: Handshake }) {
  return (
    <div className="call-shell">
      <div className="call-status" data-testid="sidecar-status">
        {statusText(handshake)}
      </div>
      <div className="call-canvas" data-testid="graph-canvas">
        <ReactFlow nodes={[]} edges={[]} proOptions={{ hideAttribution: true }} />
      </div>
    </div>
  );
}
