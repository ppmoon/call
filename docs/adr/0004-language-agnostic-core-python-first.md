# 语言无关核心（LSP + tree-sitter），Python 首发打磨

构图能力建立在 VS Code 的 LSP call-hierarchy API（蹭 Pylance / rust-analyzer / tsserver 的现成能力）加 Rust 侧车里的 tree-sitter 结构解析之上，核心天然多语言，而不是为单一语言做深度定制分析。Python 是第一门打磨与演示语言（目标用户 AI 工程师的主语言）；本仓库自身的 Rust/TS 靠 dogfood 顺带覆盖。

## Consequences

图的精度受各语言 LSP 能力上限约束；动态语言（Python）的调用边是近似而非保证，这是接受的代价。
