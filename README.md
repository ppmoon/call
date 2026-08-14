# Call

VS Code extension that projects a real codebase into a call Graph so you can understand a program top-down and direct AI edits from the canvas.

M0 is the skeleton: Sidecar handshake over stdio JSON-RPC, plus an empty Graph canvas that shows the Sidecar version.

## Layout

- `sidecar/` — Rust Sidecar (JSON-RPC over stdio, rust-analyzer framing)
- `extension/` — VS Code plugin
- `webview/` — React Flow canvas

## Develop

```bash
cargo test --workspace
cargo build -p call-sidecar

cd webview && npm install && npm test && npm run build && cd ..
cd extension && npm install && npm test && npm run compile && cd ..
```

Then in VS Code/Cursor: Run and Debug → **Run Extension**. Command Palette → **Call: Open Graph**. The canvas should show `Sidecar 0.1.0` and no Function Nodes.
