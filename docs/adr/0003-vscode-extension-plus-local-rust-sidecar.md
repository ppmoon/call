# 形态：VS Code 插件 + 本地 Rust 侧车

产品是 VS Code 插件（webview 内跑 React Flow 画布），重活交给随插件分发的本地 Rust 二进制（Sidecar）：tree-sitter 解析、索引缓存、`.call/` 读写、执行 Harness、图模型构建，插件与侧车走 stdio JSON-RPC（rust-analyzer 模式）。LLM 调用留在 TS 插件侧——SDK 生态好、流式处理省事、离 diff/编辑器近。

## Considered Options

- **Web 应用**：画布生态最成熟，但丢掉编辑器集成和本地信任，与目标用户（开发者）的工作流脱节，否决。
- **云服务后端**：MVP 阶段信任成本和运营成本都不利，留给商业化阶段，否决。
- **Rust 编译 WASM 内嵌插件**：沙盒限制多（文件系统、进程执行），否决。

## Consequences

分发采用 platform-specific VSIX，CI 交叉编译 win / mac-arm / mac-x64 / linux 四平台，用户零感知。
