# clanzi — Linux AU05 键位映射工具

[English](README.md) · [配置说明](docs/configuration.zh.md) · [故障排查](docs/troubleshooting.zh.md) · [验证状态](docs/acceptance.zh.md)

使用 **Tauri 2、React、TypeScript 和 Rust** 构建的 Ulanzi Vibe Key（AU05）本机桌面映射工具，面向 Linux x86_64。

- 为六个控件分配按键及 Ctrl/Shift/Alt/Super 组合。
- 按钮跟随物理按下和松开，旋钮旋转发送短按。
- 支持离线编辑、本机保存和 JSON 配置导入/导出。
- 后台持续转发，提供中英文界面。

桌面界面名称为 **Olanzi**。关闭窗口后仍会转发，通过**设置 → 退出 Olanzi**退出。Linux 桌面与硬件验证尚未完成，详见[验证状态](docs/acceptance.zh.md)。

## 构建与运行

安装 Rust、Node.js 24 和 [Tauri Linux 构建依赖](https://v2.tauri.app/start/prerequisites/#linux)。Ubuntu 示例：

```sh
sudo apt install libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev libxdo-dev build-essential
npm ci
cargo fetch --locked
cargo fetch --manifest-path src-tauri/Cargo.toml --locked
make deb
sudo apt install ./artifacts/clanzi_0.1.0_amd64.deb
clanzi
```

`.deb` 安装时会自动配置权限规则并启用 uinput 模块，无需额外执行脚本。安装后重新插拔接收器，再选择**设置 → 重试连接**。以普通桌面用户运行 GUI，并退出其他 AU05 客户端。规则只为当前本地会话授权 AU05 厂商接口和 `/dev/uinput`，不开放物理键盘事件节点。

更新前先退出正在运行的实例。重新构建后可运行 `./src-tauri/target/release/clanzi`，或通过 `sudo apt install --reinstall ./artifacts/clanzi_0.1.0_amd64.deb` 重新安装软件包。

## 配合 Handy 语音输入

为 AU05 按钮分配 Handy 的录音快捷键并保存。在 Handy 中使用按住说话：按住按钮录音，松开停止。三个按键和旋钮按下均自动跟随实际按压时长，无需选择模式或设置长按阈值。共享按键行为与兼容性见[配置说明](docs/configuration.zh.md)。

## 开发与检查

```sh
make dev                 # 原生桌面开发
make demo                # 隔离演示，不访问真实设备或发送系统按键
make check               # 格式、核心静态检查、前端构建及测试
cargo clippy --manifest-path src-tauri/Cargo.toml --offline --all-targets -- -D warnings
npm exec playwright install chromium
node tools/ui-smoke.mjs
```

`make dev` 通过 npm 调用 Tauri CLI。也可在两个终端分别执行 `npm run dev` 和 `cargo run --manifest-path src-tauri/Cargo.toml`。Vite 使用 1420 端口；浏览器预览只在内存保存数据。演示成功不代表硬件兼容性已验证。

| 路径 | 用途 |
|---|---|
| `ui/` | React 编辑器、双语文案、前端校验和 IPC |
| `src/` | 配置、存储、协议、运行时、hidraw/uinput 和工作线程 |
| `src-tauri/` | 原生窗口、托盘、单实例和文件对话框 |
| `tests/`、`tools/` | 确定性测试、UI 验证与打包 |
| `packaging/` | 桌面入口、权限规则和软件包生命周期脚本 |

[参考规格](cross-platform-replication-spec.zh.md)记录上游设计基线。当前 Linux 行为及差异见[配置说明](docs/configuration.zh.md)和[验证状态](docs/acceptance.zh.md)。

## 发布版本

推送 `v主版本.次版本.修订号` 标签后，[Release 工作流](.github/workflows/release.yml)会在 Ubuntu 24.04 构建 Linux amd64 软件包，校验版本、运行核心/前端/UI 测试，再创建包含 `.deb`、`SHA256SUMS` 和自动生成说明的 GitHub Release。目前不支持预发布版本后缀。

打标签前保持 `Cargo.toml`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json` 和 `package.json` 的版本一致，按需更新并提交两个 Cargo 锁文件及 `package-lock.json`。先提交工作流和全部发布文件，再校验并为该提交打标签：

```sh
node tools/release-version.mjs v0.1.0
git tag -a v0.1.0 -m "clanzi v0.1.0"
git push origin v0.1.0
```

后续发布请替换为对应版本。也可在 **Actions → Release → Run workflow** 中填写已有标签手动运行，构建使用该标签的源码。工作流使用 GitHub 自动提供的 `GITHUB_TOKEN`，无需个人访问令牌；只有发布任务获得 `contents: write`。不会覆盖已有 Release；已发布的版本请通过新版本更新。

## 卸载

```sh
sudo apt remove clanzi
```

包管理器会自动移除随包安装的权限规则和模块开机加载配置。卸载前请退出 Olanzi；仍在运行的进程可能保留已打开的设备句柄。保留用户映射、偏好和日志，文件位置见[配置说明](docs/configuration.zh.md)。

## 致谢与许可证

感谢 **[MrCroxx/olanzi](https://github.com/MrCroxx/olanzi)** 及其贡献者提供原始 Olanzi 项目、AU05 协议研究、文档和参考应用。这些工作为本 Linux 实现奠定了基础。

clanzi 采用 [MIT 许可证](LICENSE)。上游版权及许可证声明保留在 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)。
