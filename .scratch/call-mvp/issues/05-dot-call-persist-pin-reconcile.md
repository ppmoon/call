# 05 — `.call/` 持久化 v1：graph.json + Pin + Reconcile

**What to build:** 开发者拖动节点后位置被 Pin；关掉再打开窗口，布局复原。符号身份以限定名为主键，加载时 Reconcile 把元数据绑回代码——后续 Prompt History 与 Node Run 参数记忆都挂在这套骨架上。

**Blocked by:** 02 — Entry 探测 → 首张 Graph

**Status:** ready-for-agent

- [ ] 拖动节点即 Pin；自动布局不再移动被钉住的节点；未钉住的节点仍随 ELK 重排
- [ ] 布局与 Pin 写入仓库内 `.call/` 的 graph.json，随工程一起版本化
- [ ] 重载窗口后 Pin 与布局复原；删除 `.call/` 后图仍能从代码重建（仅丢失布局与历史）
- [ ] Reconcile：限定名命中则绑定；改名/移动后按位置或签名模糊匹配兜底，失配时元数据降级为未绑定而非崩溃
- [ ] 符合 ADR-0005：每符号将来一文件的目录约定在此立好（本票不必写入 Prompt History）
