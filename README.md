AwesomeProxy 🚀

The local-first AI gateway bridging the gap between any LLM and your favorite AI coding agents.

📖 简介 (About)

AwesomeProxy 是一款专为 macOS 优化的轻量级本地 AI 代理网关。它诞生的初衷是为了解决 DeepSeek 等强大模型与 Codex / Claude Code 等编程助手之间接口协议不兼容的痛点。

基于 Tauri 构建原生系统托盘 UI，内置封装好的 LiteLLM 协议转换引擎。你只需在状态栏轻轻一点，即可实现底层模型协议的无感切换与深度适配。

✨ 核心特性 (Features)

🛠 协议级抹平：完美模拟 OpenAI v1/responses (Codex 专属) 与 Anthropic v1/messages (Claude) 接口。

🖱️ 极简托盘交互：常驻 macOS 状态栏，一键切换路由策略，告别繁琐的命令行配置。

⚡️ 本地优先 (Local-first)：零云端依赖，无需数据库。所有状态基于本地 config.yaml 驱动，隐私绝对安全。

📦 免环境运行：内置 Python 独立打包的 LiteLLM Sidecar，彻底告别环境污染。

🏗 架构总览 (Architecture)

Frontend: React + TypeScript + Vite (专注于极简配置面板)

Core: Tauri / Rust (负责 YAML 读写与子进程守护)

Engine: LiteLLM (通过 PyInstaller 打包的独立二进制代理)

🚀 快速开始 (Quick Start)

(供开发者本地运行使用)

环境准备: 确保已安装 Node.js, Rust 环境。

克隆仓库:

git clone https://github.com/your-org/AwesomeProxy.git
cd AwesomeProxy


安装依赖与运行:

npm install
npm run tauri dev


🤝 鸣谢 (Acknowledgments)

UI 交互灵感及部分基础理念致敬了出色的开源项目 cc-switch。

底层核心协议转换由 LiteLLM 强力驱动。

📄 许可协议

本项目基于 MIT License 开源。
