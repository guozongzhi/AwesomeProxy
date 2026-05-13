# AwesomeProxy 🚀

The local-first AI gateway bridging the gap between any LLM and your favorite AI coding agents.

## 📖 简介 (About)

AwesomeProxy 是一款专为 macOS 优化的轻量级本地 AI 代理网关。它诞生的初衷是为了解决 DeepSeek 等强大模型与 Codex / Claude Code 等编程助手之间接口协议不兼容的痛点。

基于 Tauri 构建原生系统托盘 UI，内置封装好的 LiteLLM 协议转换引擎。你只需在状态栏轻轻一点，即可实现底层模型协议的无感切换与深度适配。

## ✨ 核心特性 (Features)

- 🛠 协议级抹平：模拟 OpenAI v1/responses 与 Anthropic v1/messages 接口。
- 🖱️ 极简托盘交互：常驻 macOS 状态栏，通过托盘打开配置面板或退出应用。
- ⚡️ 本地优先 (Local-first)：零云端依赖，无数据库；所有状态基于本地 `config.yaml` 驱动。
- 📦 免环境运行：内置 Python 独立打包的 LiteLLM Sidecar，避免污染本地 Python 环境。

## 🏗 架构总览 (Architecture)

- Frontend: React + TypeScript + Vite，负责极简配置面板。
- Core: Tauri / Rust，负责 YAML 读写、系统托盘、Sidecar 子进程守护。
- Engine: LiteLLM，通过 PyInstaller 打包为独立二进制代理 Sidecar。

## 📁 项目结构

```text
AwesomeProxy/
├── package.json
├── src/
│   ├── components/
│   ├── App.tsx
│   └── main.tsx
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── binaries/
│   └── src/
│       ├── commands.rs
│       ├── lib.rs
│       └── main.rs
└── README.md
```

## ⚙️ 配置约束

AwesomeProxy 不使用 SQLite 或任何数据库。唯一真实状态源是：

```text
~/.awesomeproxy/config.yaml
```

前端保存表单后会调用 Tauri Command，Rust Core 将配置序列化为 YAML 并覆写该文件，然后尝试调用本地 LiteLLM reload 接口。

## 🧩 LiteLLM Sidecar

开发和打包前，将预编译的 Sidecar 放到：

```text
src-tauri/binaries/litellm-sidecar
```

并确保可执行：

```bash
chmod +x src-tauri/binaries/litellm-sidecar
```

应用启动时会尝试运行：

```bash
litellm-sidecar --config ~/.awesomeproxy/config.yaml --port 4000
```

如果二进制不存在，配置面板仍可启动，但 Sidecar 状态会显示离线。应用退出时会杀死已托管的 Sidecar 子进程，释放端口。

## 🚀 快速开始 (Quick Start)

环境准备：确保已安装 Node.js、npm、Rust 和 Cargo。

```bash
npm install
npm run tauri:dev
```

前端单独开发：

```bash
npm run dev
```

构建前端：

```bash
npm run build
```

打包桌面应用：

```bash
npm run tauri:build
```

## 🤝 鸣谢 (Acknowledgments)

UI 交互灵感及部分基础理念致敬了出色的开源项目 cc-switch。

底层核心协议转换由 LiteLLM 强力驱动。

## 📄 许可协议

本项目基于 MIT License 开源。
