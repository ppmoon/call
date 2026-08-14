# 08 — Ghost Node 生成新函数

**What to build:** 开发者在画布上放下一个 Ghost Node、写提示词、把它接到现有流程上；AI 生成新函数并插入调用，经 Diff Gate 后 Ghost 转正为 Function Node，图与代码同时多出一个真实符号。

**Blocked by:** 07 — 编辑管线：节点 prompt → Diff Gate → 落盘 + Prompt History

**Status:** ready-for-agent

- [ ] 可从画布创建 Ghost Node，仅含提示词、尚无代码符号
- [ ] 将 Ghost 用 Call Edge 接到已有 Function Node 后发起生成，产出新函数定义 + 调用方插入，一次 Diff Gate 审阅
- [ ] accept 后 Ghost 转正为 Function Node（限定名绑定、Prompt History 落盘）；reject 后 Ghost 仍在，代码不变
- [ ] 未接线的 Ghost 不能生成（提示需要至少一条调用边以确定插入位置）
