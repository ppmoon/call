# 11 — Program Run + 执行高亮

**What to build:** 从 Entry 跑整段程序（Program Run）；跑完后，实际被执行到的 Function Node 高亮着色，开发者能对照 Graph 看出这条路径走了哪些节点。

**Blocked by:** 10 — Node Run：Harness + 参数面板 + 内联输出

**Status:** ready-for-agent

- [ ] 从当前 Entry 启动 Program Run，复用 10 的 runner 与本机解释器
- [ ] 运行结束后，被命中的 Function Node 与对应 Call Edge 高亮；未命中的保持默认样式
- [ ] 运行失败时高亮停在失败节点，并在该节点内联报错
- [ ] 本票不做边上流式动画
