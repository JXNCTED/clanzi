# 发布指南

[English](releases.md) · [README](../README.zh.md) · [贡献指南](../CONTRIBUTING.zh.md)

## 打标签前

推送 `v主版本.次版本.修订号` 标签后，[Release 工作流](https://github.com/JXNCTED/clanzi/blob/main/.github/workflows/release.yml)会在 Ubuntu 22.04 构建 Linux amd64 软件包，校验版本、运行核心／前端／界面测试，再创建包含 `.deb`、`.AppImage`、`70-clanzi.rules`、`clanzi.conf`、`SHA256SUMS` 和自动生成说明的 GitHub Release。目前不支持预发布版本后缀。

1. 保持 `Cargo.toml`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json` 和 `package.json` 版本一致，按需更新两个 Cargo 锁文件及 `package-lock.json`。
2. 执行[贡献指南](../CONTRIBUTING.zh.md#修改与验证)中的检查、`make deb` 和 `make appimage`，检查软件包版本、架构及打包文件。AppImage 构建会下载 Tauri 打包工具，并以解包模式运行辅助程序，构建时无需 FUSE 挂载。
3. 打标签前提交工作流和全部发布文件，不包含构建产物、日志、凭据或本机配置。

校验并为发布提交打标签（将 `0.1.0` 替换为实际版本）：

```sh
node tools/release-version.mjs v0.1.0
git tag -a v0.1.0 -m "clanzi v0.1.0"
git push origin v0.1.0
```

## 发布与检查

也可在 **Actions → Release → Run workflow** 中填写已有标签手动运行，构建使用该标签的源码。工作流使用 GitHub 自动提供的 `GITHUB_TOKEN`，无需个人访问令牌；只有发布任务获得 `contents: write`。不会覆盖已有 Release，已发布的版本请通过新版本更新。

工作流完成后确认 Release 包含两种软件包、两个权限文件和 `SHA256SUMS`。验证所有校验和并审阅自动生成说明，补充简明的改动说明及 [AppImage 权限配置](troubleshooting.zh.md#appimage-权限配置)链接。在合适的 Linux 桌面环境中测试两种软件包，包括安装／卸载及完成一次性配置后的 AppImage 设备访问。

## 准备公开仓库

建议使用以下英文仓库简介：

> Open-source Linux key mapper for Ulanzi Vibe Key (AU05), with custom shortcuts, push-to-talk, and a bilingual desktop UI.

建议话题：`linux`、`ulanzi`、`vibekey`、`au05`、`key-mapping`、`tauri`、`rust`、`react`、`typescript`。

改变仓库可见性前检查文件与 Git 历史中的凭据或私人数据，确认包含 MIT 许可证和上游声明，并检查 README 链接及截图。改变可见性和发布版本与文档准备是独立步骤。
