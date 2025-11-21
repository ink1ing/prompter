#!/bin/bash

# Prompter 可视化Shell监控启动脚本 - 增强版

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m'

print_header() {
    clear
    echo -e "${BLUE}"
    echo "╔══════════════════════════════════════════════════════════╗"
    echo "║                  🤖 PROMPTER MONITOR                     ║"
    echo "║               可视化Shell监控启动向导                      ║"
    echo "╚══════════════════════════════════════════════════════════╝"
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

print_info() {
    echo -e "${CYAN}[INFO]${NC} $1"
}

# 检查系统环境
check_system_environment() {
    print_step "1" "检查系统环境..."

    # 检查操作系统
    local os_type=$(uname -s)
    echo "   🖥️  操作系统: $os_type"

    # 检查当前Shell
    local current_shell=$(basename "$SHELL")
    echo "   🐚 当前Shell: $current_shell"

    # 检查历史文件
    local home_dir="$HOME"
    local history_files=0

    echo "   📂 历史文件检查:"
    if [ -f "$home_dir/.bash_history" ]; then
        local size=$(stat -f%z "$home_dir/.bash_history" 2>/dev/null || stat -c%s "$home_dir/.bash_history" 2>/dev/null || echo "0")
        echo "      ✅ Bash: ~/.bash_history ($size bytes)"
        ((history_files++))
    else
        echo "      ❌ Bash: 未找到"
    fi

    if [ -f "$home_dir/.zsh_history" ]; then
        local size=$(stat -f%z "$home_dir/.zsh_history" 2>/dev/null || stat -c%s "$home_dir/.zsh_history" 2>/dev/null || echo "0")
        echo "      ✅ Zsh: ~/.zsh_history ($size bytes)"
        ((history_files++))
    else
        echo "      ❌ Zsh: 未找到"
    fi

    if [ -f "$home_dir/.local/share/fish/fish_history" ]; then
        local size=$(stat -f%z "$home_dir/.local/share/fish/fish_history" 2>/dev/null || stat -c%s "$home_dir/.local/share/fish/fish_history" 2>/dev/null || echo "0")
        echo "      ✅ Fish: ~/.local/share/fish/fish_history ($size bytes)"
        ((history_files++))
    else
        echo "      ❌ Fish: 未找到"
    fi

    if [ $history_files -eq 0 ]; then
        print_error "没有找到任何Shell历史文件"
        echo ""
        echo "请确保："
        echo "1. 你的Shell启用了历史记录功能"
        echo "2. 你已经运行过一些命令"
        exit 1
    fi

    print_success "找到 $history_files 个历史文件"
}

# 检查活跃终端
check_active_terminals() {
    print_step "2" "检测活跃终端..."

    echo "   🔍 正在扫描终端进程..."

    # 检测终端应用
    local terminal_count=0
    local terminals_found=""

    # macOS特有的终端
    if pgrep -f "Terminal.app" > /dev/null 2>&1; then
        terminals_found="$terminals_found Terminal.app"
        ((terminal_count++))
    fi

    if pgrep -f "iTerm.app" > /dev/null 2>&1; then
        terminals_found="$terminals_found iTerm.app"
        ((terminal_count++))
    fi

    # 跨平台终端
    for term in "Alacritty" "kitty" "gnome-terminal" "konsole" "xterm"; do
        if pgrep -f "$term" > /dev/null 2>&1; then
            terminals_found="$terminals_found $term"
            ((terminal_count++))
        fi
    done

    # 终端多路复用器
    if pgrep -f "tmux" > /dev/null 2>&1; then
        terminals_found="$terminals_found tmux"
        ((terminal_count++))
    fi

    if pgrep -f "screen" > /dev/null 2>&1; then
        terminals_found="$terminals_found screen"
        ((terminal_count++))
    fi

    # TTY会话
    local tty_sessions=0
    if command -v who >/dev/null 2>&1; then
        tty_sessions=$(who | wc -l | tr -d ' ')
    fi

    echo "   📊 检测结果:"
    echo "      🖥️  终端应用: $terminal_count 个 ($terminals_found)"
    echo "      💻 TTY会话: $tty_sessions 个"

    local total_terminals=$((terminal_count + tty_sessions))
    if [ $total_terminals -gt 0 ]; then
        print_success "检测到 $total_terminals 个活跃终端"
    else
        print_warning "未检测到明确的终端进程（这是正常的）"
    fi
}

# 编译项目
build_project() {
    print_step "3" "编译项目..."

    if [ ! -f "Cargo.toml" ]; then
        print_error "未找到 Cargo.toml，请在项目根目录运行此脚本"
        exit 1
    fi

    if [ ! -f "./target/release/prompter" ] || [ "src/" -nt "./target/release/prompter" ]; then
        echo "   🔨 正在编译..."
        if cargo build --release > build.log 2>&1; then
            print_success "编译完成"
        else
            print_error "编译失败，查看 build.log 获取详细信息"
            tail -10 build.log
            exit 1
        fi
    else
        print_success "二进制文件已是最新"
    fi
}

# 预览可视化界面
preview_interface() {
    print_step "4" "预览监控界面..."

    echo ""
    echo -e "${WHITE}启动后的界面预览:${NC}"
    echo ""
    echo "╔══════════════════════════════════════════════════════════╗"
    echo "║                    🤖 PROMPTER MONITOR                   ║"
    echo "║               独立终端监控模式 - Claude提示词智能收集        ║"
    echo "╚══════════════════════════════════════════════════════════╝"
    echo "⏰ 启动时间: $(date '+%Y-%m-%d %H:%M:%S')"
    echo "🔧 监控间隔: 5秒"
    echo "────────────────────────────────────────────────────────────"
    echo "📊 监控状态总览"
    echo "────────────────────────────────────────────────────────────"
    echo "🖥️  活跃终端 (示例):"
    echo "   ✅ Terminal.app"
    echo "   ✅ iTerm.app"
    echo ""
    echo "📂 监控文件 (示例):"
    echo "   ✅ ~/.zsh_history (25610 bytes)"
    echo "   ✅ ~/.bash_history (75081 bytes)"
    echo ""
    echo "🎯 目标命令模式:"
    echo "   📝 claude"
    echo "   📝 claude-code"
    echo "   📝 claude-cli"
    echo "────────────────────────────────────────────────────────────"
    echo "🚀 监控服务已就绪，等待Claude命令..."
    echo ""
    echo -e "${YELLOW}当检测到中文提示词时，会显示类似:${NC}"
    echo "📥 [14:30:45.123] 200 OK - 中文提示词已保存 | Records: 1"
    echo "   └─ 📄 Content: 帮我写一个计算斐波那契数列的Python函数..."
}

# 启动选项
show_start_options() {
    print_step "5" "启动选项..."

    echo ""
    echo "🚀 选择启动模式:"
    echo ""
    echo "1️⃣  基础监控模式 (推荐新手)"
    echo "   - 只监控和保存中文提示词"
    echo "   - 不自动上传"
    echo "   - 命令: ./target/release/prompter --shell-monitor"
    echo ""
    echo "2️⃣  自动上传模式 (推荐高级用户)"
    echo "   - 监控 + 每小时自动上传到Cloudflare"
    echo "   - 需要配置 config.toml"
    echo "   - 命令: ./target/release/prompter --auto --shell-monitor"
    echo ""
    echo "3️⃣  测试模式"
    echo "   - 显示帮助信息"
    echo "   - 命令: ./target/release/prompter --help"
    echo ""

    while true; do
        read -p "请选择模式 (1/2/3) [1]: " choice
        choice=${choice:-1}

        case $choice in
            1)
                echo ""
                print_success "启动基础监控模式..."
                echo ""
                print_info "监控已开始，在其他终端运行Claude命令："
                echo "   claude \"帮我写一个函数\""
                echo "   claude \"优化这段代码\""
                echo ""
                echo "按 Ctrl+C 停止监控"
                echo ""
                exec ./target/release/prompter --shell-monitor
                ;;
            2)
                echo ""
                print_info "检查配置文件..."
                if [ ! -f "config.toml" ]; then
                    print_warning "未找到 config.toml，需要先配置"
                    echo "运行: ./auto_start.sh 来创建配置文件"
                    exit 1
                fi
                print_success "启动自动上传模式..."
                echo ""
                echo "监控 + 自动上传已开始"
                echo "按 Ctrl+C 停止服务"
                echo ""
                exec ./target/release/prompter --auto --shell-monitor
                ;;
            3)
                echo ""
                ./target/release/prompter --help
                echo ""
                echo "使用示例:"
                echo "  ./target/release/prompter --shell-monitor"
                echo "  ./target/release/prompter --auto --shell-monitor"
                exit 0
                ;;
            *)
                print_error "无效选择，请输入 1、2 或 3"
                ;;
        esac
    done
}

# 主函数
main() {
    print_header

    check_system_environment
    echo ""

    check_active_terminals
    echo ""

    build_project
    echo ""

    preview_interface
    echo ""

    show_start_options
}

# 错误处理
trap 'print_error "脚本执行中断"; exit 1' INT TERM

# 运行主函数
main "$@"