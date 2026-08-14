# 06 — M1 预览版发布（platform VSIX）

**What to build:** 一条 CI 流水线打出 win / mac-arm / mac-x64 / linux 四份 platform-specific VSIX，内含对应 Sidecar 二进制。外部开发者能装上只读构图预览版，打开 Python 工程看见 Graph。

**Blocked by:** 03 — 下钻 + Container 折叠；04 — 规模控制：外部调用过滤 + Cluster；05 — `.call/` 持久化 v1

**Status:** ready-for-agent

- [ ] CI 交叉编译四个目标平台的 Sidecar，分别打进对应 VSIX
- [ ] 在至少一个非开发机平台上安装 VSIX 后，Sidecar 能拉起、构图可用
- [ ] 预览版范围明确为只读构图（无 AI 编辑、无 Node Run），README 写清如何安装与打开 Entry
- [ ] 符合 ADR-0003 的分发方式：用户零感知、不从 GitHub Releases 另下二进制
