@echo off
REM MessageBoxA Hook 示例编译脚本 (Windows)
REM 使用方法: build_example.bat

echo ========================================
echo 编译 MessageBoxA Hook 示例
echo ========================================
echo.

cd /d %~dp0

echo 正在检查 Rust 环境...
rustc --version
if %errorlevel% neq 0 (
    echo 错误: 未找到 Rust 编译器，请先安装 Rust
    pause
    exit /b 1
)

echo.
echo 正在编译示例...
cargo build --example messagebox_hook --target x86_64-pc-windows-msvc

if %errorlevel% neq 0 (
    echo.
    echo 编译失败！
    pause
    exit /b 1
)

echo.
echo ========================================
echo 编译成功！
echo ========================================
echo.
echo 可执行文件位置:
echo target\x86_64-pc-windows-msvc\debug\examples\messagebox_hook.exe
echo.
echo 运行示例:
echo cargo run --example messagebox_hook
echo.
pause

