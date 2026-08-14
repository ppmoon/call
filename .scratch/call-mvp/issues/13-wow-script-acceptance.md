# 13 — Wow Script 验收

**What to build:** 在一个真实、开发者认得的 Python 开源项目上，一气呵成走完六步 Wow Script：打开工程秒出 Graph → 下钻两层 → 节点提示词改码 → Diff Gate accept → Node Run 重跑验证。这是 MVP 完成标志；过程中暴露的缝隙在本票补齐，并留下可复述的演示记录。

**Blocked by:** 04 — 规模控制：外部调用过滤 + Cluster；07 — 编辑管线：节点 prompt → Diff Gate → 落盘 + Prompt History；10 — Node Run：Harness + 参数面板 + 内联输出

**Status:** ready-for-agent

- [ ] 选定并文档化一个固定的真实 Python 仓库作为演示对象（体量足以展示过滤/聚簇，小到能在一次会话里走完）
- [ ] 六步可连续完成，无需手工改配置文件或重启窗口（BYO key 事先配好除外）
- [ ] 下钻两层后图仍可读；过滤与 Cluster 在该仓库上生效
- [ ] 针对某个真实 Function Node 的提示词（例如加重试）经 Diff Gate 落入源码，Prompt History 可在 `.call/` 里看到
- [ ] 对该节点 Node Run 能用面板参数跑通，节点上看到改动后的输出或明确失败原因
- [ ] 演示步骤写成仓库内可重复的剧本说明；录一次走查（文字时间线即可）
- [ ] Ghost Node、Drag-Insert、Program Run、hello-world **不是**本票验收项
