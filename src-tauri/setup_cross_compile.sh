#!/bin/bash
# 跨平台编译环境设置脚本
# 用于在 macOS 上设置 Windows 交叉编译环境
# 使用方法: ./setup_cross_compile.sh

set -e

echo "========================================"
echo "设置跨平台编译环境"
echo "========================================"
echo ""

# 检查操作系统
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo "此脚本专为 macOS 设计"
    exit 1
fi

# 检查 Rust 环境
echo "1. 检查 Rust 环境..."
if ! command -v rustc &> /dev/null; then
    echo "错误: 未找到 Rust 编译器"
    echo "请先安装 Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

rustc --version
echo ""

# 安装 Windows 目标工具链
echo "2. 安装 Windows 目标工具链..."
WINDOWS_TARGET="x86_64-pc-windows-gnu"

if rustup target list --installed | grep -q "$WINDOWS_TARGET"; then
    echo "✓ Windows 目标工具链已安装"
else
    echo "正在安装 $WINDOWS_TARGET..."
    rustup target add "$WINDOWS_TARGET"
    echo "✓ 安装完成"
fi
echo ""

# 检查 Homebrew
echo "3. 检查 Homebrew..."
if ! command -v brew &> /dev/null; then
    echo "警告: 未找到 Homebrew"
    echo "请先安装 Homebrew: https://brew.sh"
    echo ""
    read -p "是否继续? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
else
    echo "✓ 找到 Homebrew: $(which brew)"
fi
echo ""

# 安装 MinGW 工具链
echo "4. 安装 MinGW 交叉编译工具链..."
if command -v x86_64-w64-mingw32-gcc &> /dev/null; then
    echo "✓ MinGW 工具链已安装: $(which x86_64-w64-mingw32-gcc)"
else
    echo "正在安装 MinGW 工具链..."
    echo ""
    echo "选项 1: 使用 messense/macos-cross-toolchains (推荐)"
    echo "  brew tap messense/macos-cross-toolchains"
    echo "  brew install x86_64-w64-mingw32-toolchain"
    echo ""
    echo "选项 2: 使用 mingw-w64"
    echo "  brew install mingw-w64"
    echo ""
    
    read -p "是否现在安装? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo ""
        echo "选择安装方式:"
        echo "1) messense/macos-cross-toolchains (推荐)"
        echo "2) mingw-w64"
        read -p "请选择 (1/2): " choice
        
        case $choice in
            1)
                brew tap messense/macos-cross-toolchains
                brew install x86_64-w64-mingw32-toolchain
                ;;
            2)
                brew install mingw-w64
                ;;
            *)
                echo "无效选择，跳过安装"
                ;;
        esac
    fi
fi
echo ""

# 验证安装
echo "5. 验证安装..."
echo ""

MINGW_TOOLS=("x86_64-w64-mingw32-gcc" "x86_64-w64-mingw32-ar" "x86_64-w64-mingw32-dlltool")
ALL_TOOLS_FOUND=true

for tool in "${MINGW_TOOLS[@]}"; do
    if command -v "$tool" &> /dev/null; then
        echo "✓ $tool: $(which $tool)"
    else
        echo "✗ $tool 未找到"
        ALL_TOOLS_FOUND=false
    fi
done

if [ "$ALL_TOOLS_FOUND" = false ]; then
    echo ""
    echo "警告: 部分 MinGW 工具未找到，这可能导致编译失败"
    echo "请确保安装了完整的 MinGW 工具链"
    echo ""
fi

if rustup target list --installed | grep -q "$WINDOWS_TARGET"; then
    echo "✓ Rust 目标: $WINDOWS_TARGET 已安装"
else
    echo "✗ Rust 目标: $WINDOWS_TARGET 未安装"
fi
echo ""

# 测试编译
echo "6. 测试编译..."
echo ""

cd "$(dirname "$0")"

if cargo build --example messagebox_hook --target "$WINDOWS_TARGET" 2>&1 | head -n 20; then
    echo ""
    echo "========================================"
    echo "✓ 跨平台编译环境设置成功！"
    echo "========================================"
    echo ""
    echo "现在可以使用以下命令编译 Windows 目标:"
    echo "  ./build_example.sh $WINDOWS_TARGET"
    echo "  或"
    echo "  cargo build --example messagebox_hook --target $WINDOWS_TARGET"
    echo ""
else
    echo ""
    echo "========================================"
    echo "⚠ 测试编译失败"
    echo "========================================"
    echo ""
    echo "请检查错误信息并确保:"
    echo "1. MinGW 工具链已正确安装"
    echo "2. .cargo/config.toml 配置正确"
    echo "3. 所有依赖已正确配置"
    echo ""
fi

