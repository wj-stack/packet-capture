# MessageBoxA Hook 示例

此示例演示如何使用 `min_hook_rs` 在 Windows 环境下 hook `MessageBoxA` 函数。

## 功能说明

- 使用 `min_hook_rs` 库拦截 Windows API `MessageBoxA` 的调用
- 在 hook 函数中修改消息内容
- 演示如何调用原始函数（绕过 hook）

## 依赖

- `min_hook_rs`: MinHook 的 Rust 绑定 (v2.1)
- `windows-sys`: Windows API 绑定

这些依赖仅在 Windows 平台上可用。

## 编译和运行

### Windows 平台

使用编译脚本：
```bash
# 使用批处理脚本
build_example.bat

# 或使用 PowerShell
.\build_example.bat
```

或直接使用 Cargo：
```bash
cargo build --example messagebox_hook
cargo run --example messagebox_hook
```

### macOS/Linux 平台 - 跨平台编译

在 macOS 或 Linux 上，您可以交叉编译 Windows 目标：

#### 1. 设置跨平台编译环境

首先运行设置脚本：
```bash
cd src-tauri
./setup_cross_compile.sh
```

这个脚本会：
- 检查并安装 Rust Windows 目标工具链
- 安装 MinGW 交叉编译工具链（通过 Homebrew）
- 验证安装并测试编译

#### 2. 手动安装（如果脚本失败）

```bash
# 安装 Rust Windows 目标工具链
rustup target add x86_64-pc-windows-gnu

# 安装 MinGW 工具链（使用 Homebrew）
brew tap messense/macos-cross-toolchains
brew install x86_64-w64-mingw32-toolchain

# 或者使用 mingw-w64
brew install mingw-w64
```

#### 3. 编译 Windows 目标

使用更新后的编译脚本：
```bash
# 编译 Windows 目标
./build_example.sh x86_64-pc-windows-gnu

# 或直接使用 Cargo
cargo build --example messagebox_hook --target x86_64-pc-windows-gnu
```

编译后的可执行文件位于：
```
target/x86_64-pc-windows-gnu/debug/examples/messagebox_hook.exe
```

#### 4. 在非 Windows 平台上编译当前平台

在非 Windows 平台上，示例会编译但只显示提示信息：
```bash
./build_example.sh
# 或
cargo run --example messagebox_hook
```

## 工作原理

1. 使用 `GetModuleHandleA` 和 `GetProcAddress` 获取 `MessageBoxA` 的函数地址
2. 使用 `minhook-rs` 创建 hook，将 `MessageBoxA` 重定向到自定义函数
3. 在自定义函数中：
   - 打印调用信息
   - 修改消息内容
   - 调用原始函数显示修改后的消息
4. 演示如何通过原始函数指针绕过 hook

## 跨平台编译配置

项目已配置 `.cargo/config.toml` 文件，支持跨平台编译：

```toml
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
ar = "x86_64-w64-mingw32-ar"
```

## 注意事项

- 此示例仅在 Windows 平台上有效（运行时）
- 可以在 macOS/Linux 上交叉编译 Windows 目标
- Hook 技术可能被某些安全软件检测为可疑行为
- 请确保在合法和安全的环境中使用
- 在实际应用中，需要正确处理错误和资源清理
- 交叉编译需要安装相应的工具链（MinGW）

## 示例输出

在 Windows 上运行示例后，您会看到：
1. Hook 初始化信息
2. 调用 `MessageBoxA` 时，控制台会显示 hook 拦截信息
3. 弹出的消息框会显示被修改后的内容
4. 再次调用原始函数会显示原始内容

## 故障排除

### 交叉编译问题

如果遇到链接错误：
1. 确保 MinGW 工具链已正确安装
2. 检查 `.cargo/config.toml` 配置是否正确
3. 尝试重新安装工具链：`brew reinstall x86_64-w64-mingw32-toolchain`

### API 兼容性问题

如果遇到 API 调用错误：
1. 确保使用的是 `windows-sys` 而不是 `winapi`
2. 检查 `min_hook_rs` 版本是否为 2.1
3. 参考 `windows-sys` 文档确认正确的 API 用法

