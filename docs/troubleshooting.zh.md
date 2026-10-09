# 故障排查

[English](troubleshooting.md)

📚 [README](../README.zh.md) · [验收记录](acceptance.zh.md) · **故障排查**

## 接收器和输出是两种权限

`PERMISSION_REQUIRED` 附带问题区域：`receiver` 表示 AU05 厂商 hidraw 节点，`output` 表示 `/dev/uinput`。安装软件包中的规则，重新插拔接收器，以正常登录的桌面用户重试。SSH 会话可能没有 logind 的活动席位 ACL。不要将 `/dev/input/*` 开放给所有人，也不要以 root 运行界面。

目标接收器使用 VID `0xFFF1`、PID `0x00DD`、厂商 Usage Page `0xFFFC` 和 Report ID `0x55`。运行时核对实际报告描述符。随附 udev 规则针对已记录的 USB 接口 `03`；接口或描述符变化需要单独验证。

## 状态与恢复

| 错误 | 含义与下一步 |
|---|---|
| `DEVICE_MISSING` | 没有匹配的厂商接口。重新插拔，每两秒自动发现一次。 |
| `DEVICE_MULTIPLE` | 多个候选接收器。只保留一个后明确重试。 |
| `DEVICE_BUSY` | 无法取得协作式接口锁。退出其他客户端后重试。 |
| `DEVICE_OFFLINE` | 有效查询明确返回状态 `0x00`。开启 AU05 后等待下一次检查。 |
| `DEVICE_STATUS_UNKNOWN` | 查询超时或返回不支持的状态，不能据此认定设备离线。 |
| `CONFIG_INVALID` | 本机或导入 JSON 无效。导入失败保留当前草稿，损坏的本机文件须明确修复保存。 |
| `CONFIG_SAVE_FAILED` | 持久化失败。检查磁盘空间和目录权限，草稿仍保留。 |
| `OUTPUT_FAILED` | 输出或释放清理失败。修复权限后重试，重新启用前会先尝试未完成的释放。 |
| `HANDOVER_FAILED` | 无法发送交接命令，不能确认控制已交还。重新连接后重试。 |
| `INPUT_OVERFLOW` | 待执行动作超过 32 项。已丢弃最新动作，请减慢输入。 |

隐藏窗口再打开后仍显示错误。日志区分明确离线、查询超时／无效状态、无效报告和输出失败。暂停停止心跳及周期查询；明确重试只进行一次查询，不解除暂停，只有“恢复转发”清除此意图。

## 同步

交接前或交接期间观察到的按住控件会被隔离，直到松开；旋转脉冲重新开始 100 ms 静默窗口。按住控件阻止同步超过三秒时，请松开全部控件，再点击明确的重新同步按钮。应用不发送测试键。

这不是完整的物理状态快照。协议没有时间戳或序列号，无法区分任意迟到的历史按下与新按下。启动前持键及迟到报告属于阻止正式验收的硬件验证项目，见验收记录。

## 开发环境故障

| 症状 | 操作 |
|---|---|
| 缺少 `webkit2gtk-4.1` 或 appindicator | 安装 Tauri Linux 构建依赖。 |
| Cargo 无法下载 | 检查网络/代理后执行 `cargo fetch --manifest-path src-tauri/Cargo.toml --locked` 下载依赖。 |
| 原生调试启动时 Vite 不可用 | 在另一个终端运行 `npm run dev`，或使用内嵌资源的发布版二进制。 |
| UI 测试缺少浏览器 | 执行 `npm exec playwright install chromium`；也可用 `PLAYWRIGHT_CHROMIUM_EXECUTABLE` 指定兼容的已有浏览器。 |
| 浏览器以 `Operation not permitted` 退出 | 在普通桌面或 CI 环境运行 UI 测试，某些沙箱会限制 Chromium IPC。 |
| GNOME 没有托盘图标 | 再次启动 Olanzi，已有实例会重新打开窗口。 |

应用通过 `dbus-monitor` 观察 logind 的 `PrepareForSleep`；监视启动失败会记录 `SLEEP_MONITOR_FAILED`。当前实现没有持有 logind 延迟抑制器，因此睡眠前清理的时序仍须真实桌面验证。
