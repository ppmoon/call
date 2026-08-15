# 10 — Node Run：Harness + 参数面板 + 内联输出

**What to build:** 开发者在 Function Node 上打开 JSON 参数面板、点运行，Sidecar 生成 Harness、用本机 Python 解释器执行该函数，输出或报错显示在节点上。最近一次参数记进 `.call/`。这是 Wow Script 里「改完当场验证」那一步。

**Blocked by:** 05 — `.call/` 持久化 v1：graph.json + Pin + Reconcile

**Status:** done

- [x] 节点上有 JSON 参数面板；缺参或非法 JSON 时不启动运行并指出问题
- [x] Sidecar 生成 Harness，用官方 Python 扩展解析出的解释器执行（跟随 venv/conda）
- [x] 成功输出与失败 traceback 都内联在节点上；运行中有进行中状态
- [x] 最近一次合法参数按符号写入 `.call/`，下次打开面板预填
- [x] runner 是 Sidecar 内可替换的 trait；本票只实现本机运行时，不引入容器/WASM
- [x] Harness 生成与退出码判定有不依赖 LLM 的单元测试
- [x] 符合 ADR-0008
