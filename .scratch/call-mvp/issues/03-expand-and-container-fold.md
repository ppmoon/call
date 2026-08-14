# 03 — 下钻 + Container 折叠

**What to build:** 在已有 Graph 上点击 Function Node 会懒展开其内部调用子图（Expand）；模块/类以 Container 呈现，可折叠收纳其中的函数节点。开发者可以自顶向下逐层看懂流程。

**Blocked by:** 02 — Entry 探测 → 首张 Graph

**Status:** ready-for-agent

- [ ] 点击未展开的 Function Node 懒加载下一层被调节点与 Call Edge，不重构图整张图
- [ ] 已展开节点可再收起；收起后子图从画布移除（布局上 Pin 的位置保留到重开）
- [ ] 同模块/同类的函数落在可折叠 Container 内；折叠后 Container 仍保留对外 Call Edge
- [ ] fixture golden 覆盖「展开一层后的图结构」
