#!/bin/bash

# Prompter 独立Shell监控启动脚本

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m'

print_header() {
    echo -e "${BLUE}"
    echo "🔍 Prompter 独立Shell监控模式"
    echo "================================"
    echo -e "${NC}"
}

print_step() {
    echo -e "${PURPLE}[STEP $1]${NC} $2"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查并编译项目
check_and_build() {
    print_step "1" "检查项目状态..."

    if [ ! -f "Cargo.toml" ]; then
        print_error "未找到 Cargo.toml，请在项目根目录运行此脚本"
        exit 1
    fi

    if [ ! -f "./target/release/prompter" ] || [ "src/" -nt "./target/release/prompter" ]; then
        print_step "2" "编译项目..."
        cargo build --release
        print_success "编译完成"
    else
        print_success "二进制文件已是最新"
    fi
}

# 检查Shell配置
check_shell_config() {
    print_step "3" "检查Shell历史配置..."

    local home_dir="$HOME"
    local found_history=false

    echo "检查可用的历史文件:"

    # 检查bash历史
    if [ -f "$home_dir/.bash_history" ]; then
        echo "  ✅ Bash历史: $home_dir/.bash_history"
        found_history=true
    else
        echo "  ❌ Bash历史: 不存在"
    fi

    # 检查zsh历史
    if [ -f "$home_dir/.zsh_history" ]; then
        echo "  ✅ Zsh历史: $home_dir/.zsh_history"
        found_history=true
    else
        echo "  ❌ Zsh历史: 不存在"
    fi

    # 检查fish历史
    if [ -f "$home_dir/.local/share/fish/fish_history" ]; then
        echo "  ✅ Fish历史: $home_dir/.local/share/fish/fish_history"
        found_history=true
    else
        echo "  ❌ Fish历史: 不存在"
    fi

    if [ "$found_history" = false ]; then
        print_error "没有找到任何Shell历史文件"
        echo ""
        echo "请确保："
        echo "1. 你使用的Shell启用了历史记录"
        echo "2. 已经运行过一些Claude命令"
        echo "3. 历史文件路径正确"
        exit 1
    fi

    print_success "找到可用的Shell历史文件"
}

# 配置Shell历史设置优化
optimize_shell_config() {
    print_step "4" "优化Shell历史设置..."

    local shell_name=$(basename "$SHELL")
    echo "当前Shell: $shell_name"

    case "$shell_name" in
        "bash")
            echo "建议在 ~/.bashrc 中添加以下配置以优化历史记录："
            echo ""
            echo "# 启用历史时间戳"
            echo "export HISTTIMEFORMAT='%F %T '"
            echo "# 增大历史记录数量"
            echo "export HISTSIZE=10000"
            echo "export HISTFILESIZE=20000"
            echo "# 实时保存历史"
            echo "shopt -s histappend"
            echo "export PROMPT_COMMAND=\"history -a; history -c; history -r\""
            ;;
        "zsh")
            echo "建议在 ~/.zshrc 中添加以下配置以优化历史记录："
            echo ""
            echo "# 启用历史时间戳"
            echo "setopt EXTENDED_HISTORY"
            echo "# 增大历史记录数量"
            echo "export HISTSIZE=10000"
            echo "export SAVEHIST=10000"
            echo "# 实时保存历史"
            echo "setopt INC_APPEND_HISTORY"
            echo "setopt SHARE_HISTORY"
            ;;
        "fish")
            echo "Fish Shell 默认配置已经很好，无需额外优化"
            ;;
        *)
            print_warning "未识别的Shell类型: $shell_name"
            ;;
    esac

    echo ""
    read -p "是否现在配置优化设置? (y/n) [n]: " optimize_now
    if [ "$optimize_now" = "y" ]; then
        print_warning "请手动将上述配置添加到对应的配置文件中，然后重启终端"
    fi
}

# 启动独立监控
start_monitoring() {
    print_step "5" "启动独立Shell监控..."

    echo ""
    echo "🎯 即将启动的监控特性："
    echo "  - 监控所有终端中的Claude命令"
    echo "  - 自动识别中文提示词"
    echo "  - 保存到 ./data/shell_captured_prompts.md"
    echo "  - 每5秒检查一次历史文件变化"
    echo "  - 支持bash、zsh、fish等Shell"
    echo ""

    read -p "现在启动监控? (y/n) [y]: " start_now
    start_now=${start_now:-y}

    if [ "$start_now" = "y" ]; then
        echo ""
        print_success "启动独立Shell监控模式..."
        echo "在其他终端中运行Claude命令，这里会自动捕获中文提示词"
        echo "按 Ctrl+C 停止监控"
        echo ""

        # 启动监控
        ./target/release/prompter --shell-monitor
    else
        echo ""
        echo "手动启动命令："
        echo "  ./target/release/prompter --shell-monitor"
        echo ""
        echo "结合自动上传功能："
        echo "  ./target/release/prompter --auto --shell-monitor"
    fi
}

# 主函数
main() {
    print_header

    check_and_build
    check_shell_config
    optimize_shell_config
    start_monitoring

    echo ""
    print_success "Shell监控配置完成！"
    echo ""
    echo "📋 使用提示:"
    echo "1. 在任何终端中运行 'claude' 命令"
    echo "2. 监控器会自动捕获中文提示词"
    echo "3. 查看 ./data/shell_captured_prompts.md 中的结果"
    echo "4. 使用 --auto 模式可以自动上传到Cloudflare"
}

# 错误处理
trap 'print_error "脚本执行中断"; exit 1' INT TERM

# 运行主函数
main "$@"