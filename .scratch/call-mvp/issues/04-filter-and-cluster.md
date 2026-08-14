# 04 — 规模控制：外部调用过滤 + Cluster

**What to build:** 打开真实体量的 Python 工程时图不爆炸：默认隐藏标准库与第三方调用（可开关），同一层 Function Node 超过 30 个时自动聚成 Cluster。这是 Wow Script 能在知名开源项目上站得住的前提。

**Blocked by:** 03 — 下钻 + Container 折叠

**Status:** done

- [x] 默认过滤标准库与第三方调用；画布上有开关可显示它们
- [x] 单层可见 Function Node 超过 30 时自动 Cluster；点击 Cluster 可展开为个体节点
- [x] 在一个中等体量的真实 Python 仓库上，从 Entry 起 2 层的初始图可交互（不卡死、节点可读）
- [x] 过滤与聚簇规则有 fixture 测试（含「刚好 30 / 31」边界）
