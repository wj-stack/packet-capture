#!/bin/bash
# MessageBoxA Hook 示例编译脚本 (Unix/Linux/macOS)
# 支持跨平台编译 Windows 目标
# 使用方法: ./build_example.sh [target]
# 示例: ./build_example.sh x86_64-pc-windows-msvc

set -e

cd "$(dirname "$0")"

# 默认目标
DEFAULT_TARGET=""
WINDOWS_TARGET="x86_64-pc-windows-msvc"

# 解析参数
TARGET="${1:-$DEFAULT_TARGET}"

echo "========================================"
echo "编译 MessageBoxA Hook 示例"
echo "========================================"
echo ""

# 检查 Rust 环境
echo "正在检查 Rust 环境..."
if ! command -v rustc &> /dev/null; then
    echo "错误: 未找到 Rust 编译器，请先安装 Rust"
    echo "安装命令: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

rustc --version
echo ""

# 如果是 macOS 且要编译 Windows 目标
if [[ "$OSTYPE" == "darwin"* ]] && [[ "$TARGET" == "$WINDOWS_TARGET" ]]; then
    echo "检测到 macOS 系统，准备交叉编译 Windows 目标..."
    echo ""
    
    # 检查是否已安装 Windows 目标工具链
    if ! rustup target list --installed | grep -q "$WINDOWS_TARGET"; then
        echo "正在安装 Windows 目标工具链: $WINDOWS_TARGET"
        rustup target add "$WINDOWS_TARGET"
        echo ""
    fi
    
    echo "正在编译 Windows 目标: $WINDOWS_TARGET"
    echo ""
    cargo xwin build --example messagebox_hook --target "$WINDOWS_TARGET"
    
    if [ $? -eq 0 ]; then
        echo ""
        echo "========================================"
        echo "编译成功！"
        echo "========================================"
        echo ""
        echo "可执行文件位置:"
        echo "target/$WINDOWS_TARGET/debug/examples/messagebox_hook.exe"
        echo ""
        echo "注意: 此文件需要在 Windows 系统上运行"
        echo ""
    else
        echo ""
        echo "编译失败！"
        exit 1
    fi
    
elif [[ -n "$TARGET" ]]; then
    # 编译指定目标
    echo "正在编译目标: $TARGET"
    echo ""
    
    if ! rustup target list --installed | grep -q "$TARGET"; then
        echo "正在安装目标工具链: $TARGET"
        rustup target add "$TARGET"
        echo ""
    fi
    
    cargo build --example messagebox_hook --target "$TARGET"
    
    if [ $? -eq 0 ]; then
        echo ""
        echo "========================================"
        echo "编译成功！"
        echo "========================================"
        echo ""
        echo "可执行文件位置:"
        echo "target/$TARGET/debug/examples/messagebox_hook"
        echo ""
    else
        echo ""
        echo "编译失败！"
        exit 1
    fi
    
else
    # 编译当前平台
    echo "正在编译当前平台目标..."
    echo ""
    echo "注意: 此示例仅在 Windows 平台上可用"
    echo "在非 Windows 平台上编译会生成一个仅显示提示信息的程序"
    echo ""
    
    cargo build --example messagebox_hook
    
    if [ $? -eq 0 ]; then
        echo ""
        echo "========================================"
        echo "编译成功！"
        echo "========================================"
        echo ""
        echo "可执行文件位置:"
        echo "target/debug/examples/messagebox_hook"
        echo ""
        echo "运行示例:"
        echo "cargo run --example messagebox_hook"
        echo ""
        echo "要编译 Windows 目标，请使用:"
        echo "./build_example.sh $WINDOWS_TARGET"
        echo ""
    else
        echo ""
        echo "编译失败！"
        exit 1
    fi
fi

