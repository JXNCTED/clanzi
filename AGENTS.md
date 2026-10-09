# clanzi 开发约定

## 项目与范围

- clanzi 是 AU05 的 Linux 桌面键位映射工具，界面名称为 Olanzi。
- 使用 Tauri 2、React、TypeScript 和 Rust；当前不使用 Qt，不实现 macOS。
- 先读 `README.zh.md`、`docs/hold-mode.zh.md` 和 `docs/acceptance.zh.md`。原始规格位于 `cross-platform-replication-spec.zh.md`，参考项目位于 `../olanzi`（若存在）。
- 用户最新明确要求优先于原始规格：三个按钮和旋钮按下直接跟随物理按下/松开，不提供点按/长按模式选择，不等待长按阈值。旋转方向仍发送定时点按。

## 代码位置

- `ui/`：键位编辑、草稿、双语界面和 Tauri 通信；`domain.ts` 包含前端配置校验。
- `src/config.rs`、`src/storage.rs`：权威配置校验、原子保存与损坏文件备份。
- `src/protocol.rs`：AU05 报告编解码和允许发送的命令。
- `src/runtime.rs`：设备状态、物理边沿、旋转点按队列、持键所有权和释放清理。
- `src/linux.rs`：hidraw 设备识别与 uinput 输出。
- `src/service.rs`：后台工作线程、草稿、保存与生命周期协调。
- `src-tauri/`：原生窗口、托盘、单实例与文件对话框；使用独立 Cargo 清单和锁文件。
- `tests/`、`tools/ui-smoke.mjs`：核心、前端和浏览器验证。
- `packaging/`、`tools/package-deb.sh`：Linux 权限规则和 Debian 打包。

## 修改原则

- 先说明假设和验证方式，只修改当前任务必要的代码；不做顺带重构。
- 不引入未经要求的功能、依赖或抽象；保留用户已有修改。
- 草稿编辑不改变正在运行的映射；成功保存后才替换。导入只修改草稿，导出使用已保存配置。
- 普通版本 1 按钮映射直接支持实际按压时长；保留上一版版本 2 `hold` 配置兼容性。格式规则见 `docs/hold-mode.zh.md`。
- 共享输出键必须等最后一个使用者释放；暂停、断连、配置替换和退出等停止路径必须清理持键。释放失败保留清理债务，不能假装成功。
- 保持协议命令白名单，不添加设备固件、灯光、板载键表写入或普通键盘监听。
- 界面文案同步更新 `ui/i18n.ts` 的中英文；用户文档同步更新对应语言文件。
- 演示模式不得访问真实设备或发送系统按键。运行时不记录普通键盘文本或完整配置。

## 开发与验证

使用 Node.js 24 和 Rust 工具链；Linux 原生依赖见 README。保留 `package-lock.json` 和两个 `Cargo.lock`。

本工作区的 shell 命令使用 `rtk` 前缀；没有对应封装的命令使用 `rtk proxy`。

```sh
rtk proxy make dev
rtk proxy make demo
rtk proxy make check
rtk proxy cargo clippy --manifest-path src-tauri/Cargo.toml --offline --all-targets -- -D warnings
rtk proxy node tools/ui-smoke.mjs
rtk proxy make deb
```

- 按改动选择验证：运行时和配置改动检查核心测试，前端改动检查类型、构建及相关交互，原生集成改动检查桌面 crate。
- 文档等低风险修改无需添加实现镜像测试。行为测试应覆盖真实边界和失败路径。
- `make check` 包含格式、核心 Clippy、前端构建及核心/前端测试；浏览器测试需要 Playwright 浏览器，真实桌面和 AU05 测试需要相应环境。
- 不把模拟测试、编译成功或脚本存在当作硬件/界面验证通过。环境阻塞与未验证项如实记录在 `docs/acceptance*.md`。
- 关闭窗口仍会后台运行；验证新版前从设置或托盘退出旧实例。普通使用不以 root 启动 GUI。

## Git 提交

- 提交前检查变更及暂存列表，不提交依赖目录、构建产物、日志、密钥或本机用户配置；遵守 `.gitignore`。
- 提交说明描述实际改动；修订提交时保留现有主题、正文和 trailers，除非用户要求修改。
- 所有代用户创建或修订的提交必须包含：

```text
Co-authored-by: Codex <noreply@openai.com>
```
