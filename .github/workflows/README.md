# GitHub Actions CI/CD 配置说明

本项目配置了自动化的 CI/CD 流程，包括持续集成和自动发布功能。

## 工作流说明

### 1. CI 工作流 (`ci.yml`)

**触发条件：**
- 推送到 `main` 或 `develop` 分支
- 创建 Pull Request 到 `main` 或 `develop` 分支

**执行任务：**
- **代码检查**：检查前端代码格式和 Rust 代码格式
- **构建测试**：在多个平台（Windows、macOS、Linux）上测试构建是否成功

### 2. Release 工作流 (`release.yml`)

**触发条件：**
- 推送以 `v` 开头的标签（例如：`v1.0.0`）
- 手动触发（在 GitHub Actions 页面）

**执行任务：**
- 在多个平台构建 Tauri 应用
- 自动创建 GitHub Release
- 上传构建产物到 Release

## 使用方法

### 创建新版本发布

1. **更新版本号**：
   ```bash
   # 更新 package.json 中的版本号
   # 例如：从 0.1.0 更新到 0.2.0
   ```

2. **提交更改**：
   ```bash
   git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json
   git commit -m "chore: bump version to 0.2.0"
   git push
   ```

3. **创建标签并推送**：
   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```

4. **自动构建和发布**：
   - 推送标签后，GitHub Actions 会自动触发构建流程
   - 构建完成后，会在 GitHub Releases 页面创建新的 Release
   - 所有平台的构建产物会自动上传到 Release

### 手动触发发布

1. 访问 GitHub 仓库的 Actions 页面
2. 选择 "Release" 工作流
3. 点击 "Run workflow" 按钮
4. 选择分支并点击 "Run workflow"

## 支持的平台

- **Windows**: x86_64-pc-windows-msvc
- **macOS**: 
  - aarch64-apple-darwin (Apple Silicon)
  - x86_64-apple-darwin (Intel)
- **Linux**: x86_64-unknown-linux-gnu

## 注意事项

1. **子模块**：工作流会自动初始化 `GameProxySnifferPro` 子模块
2. **签名密钥**：如果需要代码签名，需要在 GitHub Secrets 中配置：
   - `TAURI_PRIVATE_KEY`: Tauri 私钥
   - `TAURI_KEY_PASSWORD`: 私钥密码
3. **权限**：确保 GitHub Actions 有写入仓库内容的权限

## 故障排查

### 构建失败

1. 检查 Actions 日志，查看具体错误信息
2. 确认所有依赖都已正确安装
3. 检查子模块是否正确初始化

### Release 未创建

1. 确认标签格式正确（以 `v` 开头）
2. 检查是否有构建错误
3. 确认 GitHub Token 权限正确

