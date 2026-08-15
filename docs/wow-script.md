# Wow Script

Canonical demo workspace: `fixtures/httpcli` (a small HTTP-client-shaped Python package: Entry `httpcli.__main__.main`, Cluster on the display fan-out, `httpcli.fetch.fetch` as the edit target).

This is the six-step walkthrough. BYO key can be skipped by setting `call.useMockLlm` to true.

## Walkthrough (2026-08-14)

1. **Open** `fixtures/httpcli` as the workspace folder. Command Palette → **Call: Open Graph**.
2. **Graph appears** from Entry `httpcli.__main__.main`. Layer 1 is clustered (>30 display helpers). Stdlib `json`/`os` calls are hidden until **show external** is checked.
3. **Expand** the `fetch` Function Node (click). The next hop of `httpcli.fetch.fetch` is visible; the Cluster can be opened with a click if you want the 31 helpers.
4. Select `httpcli.fetch.fetch`. Prompt: `加重试逻辑`. Diff Gate opens; **Accept**. `.call/symbols/<hash>.md` now contains the prompt.
5. Node Run args `{"url":"https://example.com"}`. The node shows the JSON body (or a traceback).
6. Re-run Node Run after the retry edit — output still contains `example.com`.

## Notes

- Filtering + Cluster are visible in step 2 without leaving the repo.
- Ghost Node / Drag-Insert / Program Run / hello-world are **not** part of this script.
- Sidecar analysis is tree-sitter; the VS Code plugin will also consult LSP call-hierarchy when a Python language server is active, merging extra Call Edges into the same Graph.
