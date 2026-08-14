import { useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import type { Handshake, HandshakeMessage } from "./handshake";
import "./main.css";

declare function acquireVsCodeApi(): {
  postMessage(message: unknown): void;
};

const vscode = acquireVsCodeApi();

function Host() {
  const [handshake, setHandshake] = useState<Handshake>({ status: "connecting" });

  useEffect(() => {
    const onMessage = (event: MessageEvent<HandshakeMessage>) => {
      const data = event.data;
      if (!data || data.type !== "handshake") {
        return;
      }
      if (data.error) {
        setHandshake({ status: "error", message: data.error });
        return;
      }
      if (data.version) {
        setHandshake({ status: "ok", version: data.version });
      }
    };
    window.addEventListener("message", onMessage);
    vscode.postMessage({ type: "ready" });
    return () => window.removeEventListener("message", onMessage);
  }, []);

  return <App handshake={handshake} />;
}

const root = document.getElementById("root");
if (root) {
  createRoot(root).render(<Host />);
}
