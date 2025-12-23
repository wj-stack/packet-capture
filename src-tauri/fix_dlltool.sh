#!/bin/bash
# 修复 dlltool 缺失问题的脚本
# 使用方法: ./fix_dlltool.sh

set -e

echo "========================================"
echo "修复 MinGW dlltool 缺失问题"
echo "========================================"
echo ""

cd "$(dirname "$0")"

# 检查是否已安装 MinGW 工具链
if ! command -v x86_64-w64-mingw32-gcc &> /dev/null; then
    echo "错误: 未找到 MinGW 工具链"
    echo ""
    echo "请先安装 MinGW 工具链:"
    echo ""
    echo "选项 1: 使用 messense/macos-cross-toolchains (推荐)"
    echo "  brew tap messense/macos-cross-toolchains"
    echo "  brew install x86_64-w64-mingw32-toolchain"
    echo ""
    echo "选项 2: 使用 mingw-w64"
    echo "  brew install mingw-w64"
    echo ""
    exit 1
fi

echo "1. 检查已安装的 MinGW 工具..."
echo ""

# 查找 MinGW 工具的位置
MINGW_GCC=$(which x86_64-w64-mingw32-gcc)
MINGW_BIN_DIR=$(dirname "$MINGW_GCC")

echo "MinGW 工具目录: $MINGW_BIN_DIR"
echo ""

# 检查 dlltool
if [ -f "$MINGW_BIN_DIR/x86_64-w64-mingw32-dlltool" ]; then
    echo "✓ 找到 dlltool: $MINGW_BIN_DIR/x86_64-w64-mingw32-dlltool"
else
    echo "✗ 未找到 dlltool"
    echo ""
    echo "2. 尝试查找 dlltool..."
    
    # 尝试在常见位置查找
    POSSIBLE_PATHS=(
        "/usr/local/bin/x86_64-w64-mingw32-dlltool"
        "/opt/homebrew/bin/x86_64-w64-mingw32-dlltool"
        "/usr/local/opt/mingw-w64/bin/x86_64-w64-mingw32-dlltool"
        "/opt/homebrew/opt/mingw-w64/bin/x86_64-w64-mingw32-dlltool"
    )
    
    FOUND=false
    for path in "${POSSIBLE_PATHS[@]}"; do
        if [ -f "$path" ]; then
            echo "找到 dlltool: $path"
            echo "创建符号链接..."
            ln -sf "$path" "$MINGW_BIN_DIR/x86_64-w64-mingw32-dlltool"
            echo "✓ 符号链接已创建"
            FOUND=true
            break
        fi
    done
    
    if [ "$FOUND" = false ]; then
        echo ""
        echo "未找到 dlltool，需要重新安装 MinGW 工具链"
        echo ""
        echo "请运行以下命令重新安装:"
        echo "  brew uninstall x86_64-w64-mingw32-toolchain 2>/dev/null || true"
        echo "  brew install x86_64-w64-mingw32-toolchain"
        echo ""
        echo "或者:"
        echo "  brew uninstall mingw-w64 2>/dev/null || true"
        echo "  brew install mingw-w64"
        exit 1
    fi
fi

echo ""
echo "3. 验证所有必需工具..."
echo ""

REQUIRED_TOOLS=("x86_64-w64-mingw32-gcc" "x86_64-w64-mingw32-ar" "x86_64-w64-mingw32-dlltool")
ALL_FOUND=true

for tool in "${REQUIRED_TOOLS[@]}"; do
    if command -v "$tool" &> /dev/null; then
        echo "✓ $tool: $(which $tool)"
    else
        echo "✗ $tool 未找到"
        ALL_FOUND=false
    fi
done

if [ "$ALL_FOUND" = true ]; then
    echo ""
    echo "========================================"
    echo "✓ 所有工具已就绪！"
    echo "========================================"
    echo ""
    echo "现在可以尝试编译:"
    echo "  cargo build --example messagebox_hook --target x86_64-pc-windows-gnu"
    echo ""
else
    echo ""
    echo "========================================"
    echo "✗ 仍有工具缺失"
    echo "========================================"
    echo ""
    echo "请重新安装 MinGW 工具链:"
    echo "  brew reinstall x86_64-w64-mingw32-toolchain"
    echo ""
fi

