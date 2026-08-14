# 09 — Drag-Insert 已有函数

**What to build:** 侧栏列出仓库内已有函数，拖进某条流程后，AI 在正确位置插入调用（不新建函数），经同一套 Diff Gate 落盘。这是「拖拽编码」对存量符号的那一半。

**Blocked by:** 07 — 编辑管线：节点 prompt → Diff Gate → 落盘 + Prompt History

**Status:** ready-for-agent

- [ ] 侧栏可搜索并列出当前工程的函数符号，拖到画布上的目标 Function Node / 流程位置
- [ ] 生成的修改是在调用方插入对该函数的调用，而不是复制或新建该函数
- [ ] 走与 07 相同的 Diff Gate；reject 则图上临时边消失、代码不变
- [ ] 拖到不合法目标（例如拖到第三方 Cluster）时拒绝并说明原因
- [ ] 本票不做「拖动已有 Call Edge 重排调用顺序」（二期）
