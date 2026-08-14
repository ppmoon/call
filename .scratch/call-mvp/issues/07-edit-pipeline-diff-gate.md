# 07 — 编辑管线：节点 prompt → Diff Gate → 落盘 + Prompt History

**What to build:** 开发者配置自己的 LLM API key 后，在 Function Node 上写一条提示词，得到针对该函数的修改提案，经 VS Code 原生 diff 审阅，accept 才写入代码，同时把这条提示词记入该符号的 Prompt History。拒绝则代码与 `.call/` 都不变。

**Blocked by:** 05 — `.call/` 持久化 v1：graph.json + Pin + Reconcile

**Status:** ready-for-agent

- [ ] 插件设置里能保存 BYO API key；未配置时发起编辑会明确提示，且不把代码发往任何地方
- [ ] 单次调用管线：目标函数源码 + 上下游签名 + 精选上下文一次送入模型（无 agent 循环）
- [ ] 提案走 Diff Gate：原生 diff 编辑器审阅；accept 落盘；reject 或关闭则无写入
- [ ] accept 后该符号的 Prompt History 追加一条记录（`.call/symbols/<限定名哈希>.md`）
- [ ] LLM 响应可 mock/录制回放；golden 覆盖「accept 写入 / reject 不写入」两条路径
- [ ] 符合 ADR-0006 与 ADR-0007：不过审阅门、不走 vscode.lm、零遥测
