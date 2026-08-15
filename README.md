# Call

VS Code extension that projects a real codebase into a call Graph so you can understand a program top-down and direct AI edits from the canvas.

## Install (M1 preview VSIX)

CI builds platform-specific VSIX artifacts (`call-linux-x64`, `call-win32-x64`, `call-darwin-x64`, `call-darwin-arm64`), each with the matching Sidecar binary. Install the one for your OS via **Extensions: Install from VSIX**. Open a Python folder → **Call: Open Graph**. The preview Graph is fully usable; set `call.openaiApiKey` only when you want AI edits.

## Develop

```bash
cargo test --workspace
cargo build -p call-sidecar
cd webview && npm ci && npm test && npm run build
cd ../extension && npm ci && npm test && npm run compile
```

Run and Debug → **Run Extension** → **Call: Open Graph**. Open `fixtures/httpcli` or `fixtures/python-toy` as the workspace.

Wow Script: [`docs/wow-script.md`](docs/wow-script.md)

## Layout

- `sidecar/` — Rust Sidecar (tree-sitter Graph, `.call/` meta, Harness runner)
- `extension/` — VS Code plugin (Diff Gate, BYO LLM, LSP merge)
- `webview/` — React Flow canvas
- `fixtures/` — python-toy, wide (Cluster 30/31), httpcli (Wow Script)
