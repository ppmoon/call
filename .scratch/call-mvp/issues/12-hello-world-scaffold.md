# 12 — hello-world 脚手架

**What to build:** 打开一个没有可构图代码的空工程时，提供一键脚手架：语言选择器（默认 Python）生成最小 hello-world，随即按 Entry 构图。这是「没有代码就走 hello-world」那条入口，不是 Wow Script 主线。

**Blocked by:** 02 — Entry 探测 → 首张 Graph

**Status:** done

- [x] 空工程（探测不到 Entry）时画布给出明确空态与「生成 hello-world」动作
- [x] 语言选择器默认 Python；生成后工程内有可运行的 Entry，并立即出现 Graph
- [x] 不覆盖已有源码；非空但无 Entry 时只提供手动 Entry 选择器，不擅自脚手架
- [x] Python 模板能被 02 的构图路径消费（同一套探测，不走特例解析器）
