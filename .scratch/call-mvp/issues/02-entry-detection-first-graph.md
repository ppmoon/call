# 02 — Entry 探测 → 首张 Graph

**What to build:** 打开一个带 `main` 的 Python fixture 工程，自动（或手动）选定 Entry，画布上出现从该入口出发 1~2 层的 Function Node 与 Call Edge，ELK 自动布局。这是第一次「看见真实代码变成图」。

**Blocked by:** 01 — 仓库骨架 + 三方握手

**Status:** done

- [x] 打开 Python fixture 时自动探测 main 作为 Entry；探测失败时提供手动 Entry 选择器
- [x] 构图走 LSP call-hierarchy + Sidecar tree-sitter，产出 Function Node + Call Edge，初始深度 1~2 层
- [x] ELK 分层自动布局，画布可平移缩放
- [x] fixture 仓库的图结构有 golden 快照测试；结构漂移则测试失败
- [x] 术语与 ADR-0001 / ADR-0004 一致：图是代码投影，Python 为首发打磨语言
