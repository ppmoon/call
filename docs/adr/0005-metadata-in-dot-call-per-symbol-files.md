# 元数据进仓库 `.call/`：每符号一文件，限定名 + Reconcile 绑定

图布局、Pin、Prompt History、Node Run 参数记忆全部存在仓库内 `.call/` 目录、随 git 版本化：`graph.json`（布局/钉住/参数）+ `symbols/<限定名哈希>.md`（每符号提示词历史）。每符号一文件是为了团队协作时 diff 与合并友好。符号身份以限定名为主键，加载时 Reconcile（位置/签名模糊匹配）兜底；经本工具发起的改名由工具同步元数据。

## Considered Options

- **写进代码注释**（`/// prompt: ...`）：污染源码，否决。
- **git notes**：工具链和平台支持太差，否决。
- **单个大 JSON**：团队协作必然合并冲突，否决。
