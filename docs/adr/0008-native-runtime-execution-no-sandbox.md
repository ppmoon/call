# Node Run 用本机运行时 Harness，不做容器/WASM 隔离

单节点运行 = 生成临时 Harness 直接用本机运行时执行，与开发者自己跑测试同级信任；Python 解释器从官方 Python 扩展 API 获取（自动跟随 venv/conda）。跑的是用户自己仓库的代码，隔离收益存疑，而容器的冷启动、体积、跨平台成本很高。

## Consequences

Rust 侧车里 runner 抽象成 trait；未来若出现「跑不可信代码」的场景（如分享图/云执行），再插容器实现，不推翻现有结构。
