# Olanzi Linux 版

[English](README.md) · [规格](cross-platform-replication-spec.zh.md) · [验收记录](docs/acceptance.zh.md)

使用 **Tauri 2、React、TypeScript 和 Rust** 实现的 AU05 本机键位映射工具。按本轮要求，优先实现 Linux x86_64，尚未实现 macOS。

选择六个控件之一，分配一个普通键及可选的 Ctrl/Shift/Alt/Super 修饰键，然后保存到本机。没有设备或权限时也能编辑。关闭窗口不会终止运行时，再次启动会回到原窗口并保留草稿。

## 运行

安装生成的软件包，再明确安装设备权限：

```sh
sudo apt install ./artifacts/clanzi_0.1.0_amd64.deb
sudo sh /usr/share/clanzi/install-permissions.sh
clanzi
```

安装规则后重新插拔接收器，在**设置 → 重试连接**中重新检查。日常使用请以普通桌面用户启动。连接前退出 Ulanzi Studio 和其他 AU05 客户端。规则为当前本地会话授权 AU05 厂商接口和 `/dev/uinput`，不开放物理键盘事件节点。

目标验收环境为 Ubuntu 24.04 GNOME Wayland 和 Xorg。本工作区在 Ubuntu 22.04 x86_64 上构建，这不能证明两个目标环境的硬件兼容性。实际检查及剩余限制见[验收记录](docs/acceptance.zh.md)。

## 配合 Handy 按住说话

先在旧 clanzi 实例中选择 **设置 → 退出 Olanzi**，然后启动更新后的程序或安装重新构建的软件包。关闭窗口只会隐藏程序。

AU05 按钮现在自动跟随物理按下和松开：短按就是短按，按住则保持快捷键按下，松开才释放。没有模式选项，也没有长按判断阈值。重启更新后的程序后，已有快捷键直接生效。配合 Handy 时，按需分配其录音快捷键，并在 Handy 中使用按住说话。三个按键和旋钮按下均采用此行为，旋转方向仍为点按。

已有按钮映射同样跟随实际按压时长。详见[按钮行为与配置兼容性](docs/hold-mode.zh.md)。

## 开发

安装 [Tauri Linux 构建依赖](https://v2.tauri.app/start/prerequisites/#linux)、Rust 和 Node.js 24。两个 Cargo 锁文件及 `package-lock.json` 固定依赖版本。

```sh
sudo apt install libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev libxdo-dev build-essential
npm ci
cargo fetch --locked
cargo fetch --manifest-path src-tauri/Cargo.toml --locked
make dev
```

`make dev` 通过 npm 使用 Tauri CLI。如需避免下载 CLI，可在两个终端分别执行 `npm run dev` 与 `cargo run --manifest-path src-tauri/Cargo.toml`。原生开发使用 1420 端口上的 Vite 服务。

```sh
make demo
# Alternatively, run the packaged binary:
clanzi --demo
# Browser-only, explicitly labeled preview:
npm run dev
```

原生模拟模式使用独立实例标识及临时配置目录，不打开真实接收器，也不创建虚拟键盘。浏览器预览只在内存中保留数据。演示成功不等于硬件验证通过。

## 检查与打包

```sh
make check
cargo clippy --manifest-path src-tauri/Cargo.toml --offline --all-targets -- -D warnings
npm exec playwright install chromium
node tools/ui-smoke.mjs
make deb
```

`make deb` 生成内嵌前端的 `artifacts/clanzi_0.1.0_amd64.deb`，安装后无需 Rust 或 Node 工具链。独立校验器只接受规格规定的格式：

```sh
cargo run --offline --bin clanzi-validate -- examples/keymap.json
```

## 配置与行为

- 配置位置：`$XDG_CONFIG_HOME/olanzi-cross-platform/keymap.json`；XDG 未设置或为相对路径时使用 `~/.config/olanzi-cross-platform/keymap.json`。偏好位于同目录的 `preferences.json`。
- 明确保存之前不创建配置文件。六项“不执行动作”可以保存；接管后会丢弃对应厂商事件。
- 导入完整校验后只替换草稿，导出使用已保存版本。原生文件对话框负责覆盖确认。
- 启动时保留损坏文件。用户明确修复保存时先建立唯一的 `keymap.invalid-YYYYMMDD-HHmmss-N.json` 备份。
- 保存在输入工作线程外执行，冻结整份草稿，同步临时文件后原子替换。保存失败保留草稿及旧生效配置；保存成功后的转发故障单独呈现。
- 旋转点按持续 40 ms，相邻点按至少间隔 10 ms；点按队列先进先出，最多 32 个待执行动作。所有已映射按钮直接跟随物理按下和松开事件，在物理松开或停止转发时释放，不阻塞点按队列。共用的输出键会保持按下，直到最后一个使用者松开。线程延迟不会压缩补发多个过期边沿。取消会清空队列，释放失败仍保留清理债务。
- 只允许发送已记录的在线查询、软件在线／离线交接和心跳命令，不提供设备按键表、灯效、固件或设备标识操作。
- 日志只保存在配置目录的 `logs/` 中，最多两个 5 MiB 文件。不记录普通键盘文本或映射内容。运行时不提供网络服务，不需要账号。

## 目录

| 路径 | 职责 |
|---|---|
| `ui/` | 双语编辑器、设置、草稿界面、隔离浏览器预览 |
| `src/config.rs`、`src/storage.rs` | 严格格式、重复字段拒绝、持久化、修复备份 |
| `src/protocol.rs` | 小端 TEA、精确报告边界、命令白名单 |
| `src/runtime.rs` | 就绪条件、交接、边沿过滤、队列、清理债务 |
| `src/linux.rs` | 核对描述符的 hidraw 访问和 uinput 键盘 |
| `src/service.rs` | 后台线程、异步保存、模型、日志、睡眠观察 |
| `src-tauri/` | 原生窗口、单实例、托盘、原生文件对话框 |
| `tests/` | 确定性契约测试及故障注入测试 |

## 卸载

```sh
sudo sh /usr/share/clanzi/remove-permissions.sh
sudo apt remove clanzi
```

只移除本应用的权限规则，保留用户映射、偏好和日志。如需清除这些数据，请退出 Olanzi 后明确删除配置目录。另见[故障排查](docs/troubleshooting.zh.md)。
