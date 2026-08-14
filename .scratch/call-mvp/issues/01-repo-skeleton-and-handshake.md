# 01 — 仓库骨架 + 三方握手

**What to build:** 开发者安装未发布的插件后，能启动 Sidecar、打开一张空画布，并看到侧车版本号——证明插件、webview、Sidecar 三方已连通。这是后续所有切片的地基。

**Blocked by:** None — can start immediately.

**Status:** done

- [x] 仓库是 monorepo：Rust Sidecar crate、TS 插件、webview 画布三块都能独立构建
- [x] 插件激活时拉起 Sidecar，stdio JSON-RPC ping 成功，画布显示侧车版本
- [x] 一条 VS Code 命令打开空 React Flow 画布（无节点）
- [x] CI 对三块做 build + 单元测试；失败则整条流水线红
