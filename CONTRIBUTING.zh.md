# 参与 clanzi 开发

[English](CONTRIBUTING.md) · [README](README.zh.md)

欢迎反馈问题、参与 Linux／AU05 测试、改进文档及提交范围明确的 PR。没有设备也可以体验编辑器并参与前端开发。

## 准备工作区

按[从源码构建](README.zh.md#从源码构建)安装 Node.js 24、稳定版 Rust、Linux 原生依赖及锁定的 npm／Cargo 依赖。修改行为前阅读 [AGENTS.md](https://github.com/JXNCTED/clanzi/blob/main/AGENTS.md)和[配置说明](docs/configuration.zh.md)。[参考规格](cross-platform-replication-spec.zh.md)记录上游基线，实际按压时长等当前差异以配置文档为准。

选择适合当前工作的环境：

| 命令 | 环境 | 硬件与系统输入 |
|---|---|---|
| `npm run dev` | Vite 输出的本机网址，端口 1420 | 不访问硬件，数据仅保存在内存 |
| `make demo` | 使用隔离演示状态的原生 Tauri 窗口 | 不访问真实设备或发送系统按键 |
| `make dev` | 使用真实后端的原生 Tauri 开发环境 | 可访问 AU05 并发送映射按键 |

原生模式需要桌面显示环境。`make dev` 和 `make demo` 通过 npm 调用 Tauri CLI，首次使用可能需要下载。也可在两个终端分别运行 `npm run dev` 和 `cargo run --manifest-path src-tauri/Cargo.toml -- --demo` 来启动原生演示。

真实设备开发时，先安装 `.deb` 配置权限，重新插拔接收器，并在启动开发版本前退出已安装的实例。以普通桌面用户运行，关闭其他 AU05 客户端。关闭窗口后进程仍在运行，请通过设置或托盘退出。

## 找到相关代码

| 路径 | 职责 |
|---|---|
| `ui/` | React 编辑器、草稿、双语文案、前端校验与 IPC |
| `src/config.rs`、`src/storage.rs` | 权威校验、原子保存、损坏文件备份 |
| `src/protocol.rs` | AU05 报告与允许发送的命令 |
| `src/runtime.rs` | 物理边沿、旋转队列、共享输出键与清理 |
| `src/linux.rs`、`src/service.rs` | hidraw／uinput、工作线程、保存与生命周期 |
| `src-tauri/` | 原生窗口、托盘、单实例与文件对话框 |
| `tests/`、`tools/` | 核心／前端测试、浏览器检查、版本校验与打包 |
| `packaging/` | 桌面入口、权限与软件包生命周期脚本 |

## 修改与验证

聚焦当前问题，保留草稿与已保存映射的边界、共享按键所有权、停止路径清理及设备命令白名单。界面文案与用户文档同步更新中英文，不记录普通键盘文本或完整映射。

代码改动运行：

```sh
make check
cargo clippy --manifest-path src-tauri/Cargo.toml --offline --all-targets -- -D warnings
```

`make check` 包含格式、核心 Clippy、TypeScript／构建及核心／前端／版本／权限脚本测试。桌面 crate 使用独立 Cargo 清单和锁文件，需单独检查 Clippy。

界面改动还需运行：

```sh
npm exec playwright install chromium
node tools/ui-smoke.mjs
```

界面检查使用 `make check` 或 `npm run build` 生成的生产资源 `dist/`。可通过 `PLAYWRIGHT_CHROMIUM_EXECUTABLE` 指定兼容的现有浏览器。浏览器检查不能证明原生 WebKit、托盘或硬件兼容性。

打包改动运行 `make deb` 和 `make appimage`，检查两种软件包。AppImage 构建会下载打包工具，发布构建使用 Ubuntu 22.04。运行时改动添加覆盖真实边界或失败路径的行为测试。仅修改文档时检查链接和操作说明，无需增加实现镜像测试。

## 反馈问题或硬件测试结果

先检查[故障排查](docs/troubleshooting.zh.md)和已有 [Issues](https://github.com/JXNCTED/clanzi/issues)，并提供：

- clanzi 版本或提交、安装方式、Linux 发行版与内核版本。
- 桌面环境及 Wayland／Xorg 会话；浏览器演示、原生演示还是真实硬件。
- 可获得的 AU05 固件版本、接收器数量及是否关闭其他 AU05 客户端。
- 复现步骤、预期／实际行为和界面状态／错误码。
- 移除隐私信息后的相关诊断日志片段；路径见[配置说明](docs/configuration.zh.md)。

欢迎使用中文或英文。

## 提交 PR

说明问题、修改后的行为及已执行检查，明确记录无法执行的验证。不要提交构建产物、依赖目录、日志、凭据或本机用户配置。保留 `package-lock.json` 和两个 Cargo 锁文件，仅在依赖实际改变时更新。

发布维护者请遵循[发布指南](docs/releases.zh.md)。
