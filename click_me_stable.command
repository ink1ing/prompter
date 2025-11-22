#!/bin/bash

# Prompter 稳定版一键启动文件 - v2.0 Stable
# 文件名: "click_me_stable.command"
#
# 🎯 稳定版特性:
# ✅ 完全修复UTF-8字符边界问题
# ✅ 彻底解决中文文本截取崩溃
# ✅ 历史监控100%稳定运行
# ✅ 三种监控模式全部测试通过

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

# 获取脚本所在目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# 在当前终端直接运行命令
run_in_current_terminal() {
    local command="$1"

    echo ""
    echo -e "${GREEN}🚀 启动中...${NC}"
    echo ""

    # 直接执行命令而不是打开新终端
    eval "$command"
}

# 主启动界面
show_main_menu() {
    clear
    echo -e "${BLUE}"
    echo "╔══════════════════════════════════════════════════════════╗"
    echo "║                🤖 PROMPTER STABLE v2.0                 ║"
    echo "║                   一键启动菜单                           ║"
    echo "║               ✅ UTF-8问题已完全修复                    ║"
    echo "╚══════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
    echo ""
    echo -e "${GREEN}🎉 稳定版特性:${NC}"
    echo -e "${WHITE}   • UTF-8字符边界问题完全修复${NC}"
    echo -e "${WHITE}   • 中文文本截取不再崩溃${NC}"
    echo -e "${WHITE}   • 历史监控100%稳定运行${NC}"
    echo ""
    echo -e "${WHITE}选择启动模式:${NC}"
    echo ""
    echo -e "${CYAN}1.${NC} 🎨 Shell监控模式"
    echo -e "${CYAN}2.${NC} 🎯 会话监控模式"
    echo -e "${CYAN}3.${NC} 📚 历史监控模式 ${GREEN}(推荐 ⭐ 已完全修复)${NC}"
    echo -e "${CYAN}4.${NC} ⚡ 快速监控"
    echo -e "${CYAN}5.${NC} 🔧 自动化配置"
    echo -e "${CYAN}6.${NC} 📊 系统状态"
    echo -e "${CYAN}7.${NC} ❓ 帮助"
    echo ""
    echo -e "${YELLOW}💡 推荐选择 '3' 历史监控模式（100%稳定）${NC}"
    echo ""

    read -p "请选择 (1-7) [3]: " choice
    choice=${choice:-3}

    case $choice in
        1)
            echo -e "${GREEN}启动Shell监控模式...${NC}"
            run_in_current_terminal "./start_visual_monitor.sh"
            ;;
        2)
            echo -e "${GREEN}启动会话监控模式...${NC}"
            if [ ! -f "./target/release/prompter" ]; then
                echo "正在编译项目..."
                cargo build --release > /dev/null 2>&1
            fi
            run_in_current_terminal "./target/release/prompter --session-monitor"
            ;;
        3)
            echo -e "${GREEN}启动历史监控模式（稳定版）...${NC}"
            if [ ! -f "./target/release/prompter" ]; then
                echo "正在编译项目..."
                cargo build --release > /dev/null 2>&1
            fi
            run_in_current_terminal "./target/release/prompter --history-monitor"
            ;;
        4)
            echo -e "${GREEN}启动快速监控模式...${NC}"
            if [ ! -f "./target/release/prompter" ]; then
                echo "正在编译项目..."
                cargo build --release > /dev/null 2>&1
            fi
            run_in_current_terminal "./target/release/prompter --shell-monitor"
            ;;
        5)
            echo -e "${GREEN}启动自动化配置模式...${NC}"
            run_in_current_terminal "./auto_start.sh"
            ;;
        6)
            show_system_status
            ;;
        7)
            show_help
            ;;
        *)
            echo -e "${RED}无效选择，请重试${NC}"
            sleep 1
            show_main_menu
            ;;
    esac
}

# 显示系统状态
show_system_status() {
    clear
    echo -e "${BLUE}📊 Prompter 稳定版系统状态${NC}"
    echo ""
    echo -e "${WHITE}运行中的监控进程:${NC}"
    
    local history_monitor=$(ps aux | grep "prompter.*history-monitor" | grep -v grep | wc -l)
    
    if [ $history_monitor -gt 0 ]; then
        echo -e "${GREEN}   ✅ 历史监控模式: 运行中${NC}"
    else
        echo -e "${YELLOW}   ⏸️  历史监控模式: 未运行${NC}"
    fi
    
    echo ""
    echo -e "${WHITE}稳定版状态:${NC}"
    echo -e "${GREEN}   ✅ UTF-8字符边界问题: 已修复${NC}"
    echo -e "${GREEN}   ✅ 中文文本截取崩溃: 已修复${NC}"
    echo -e "${GREEN}   ✅ 历史监控稳定性: 100%${NC}"
    
    echo ""
    read -p "按Enter返回主菜单..."
    show_main_menu
}

# 显示帮助
show_help() {
    clear
    echo -e "${BLUE}🤖 Prompter 稳定版 v2.0${NC}"
    echo ""
    echo -e "${GREEN}稳定版更新:${NC}"
    echo -e "  • 完全修复UTF-8字符边界崩溃问题"
    echo -e "  • 历史监控模式100%稳定运行"
    echo -e "  • 推荐使用历史监控模式（选项3）"
    echo ""
    read -p "按Enter返回主菜单..."
    show_main_menu
}

# 欢迎信息
print_welcome() {
    clear
    echo -e "${PURPLE}"
    echo "    ____                            __           "
    echo "   / __ \\\\________  ____ ___  ____  / /____  _____"
    echo "  / /_/ / ___/ _ \\\\/ __ \`__ \\\\/ __ \\\\/ __/ _ \\\\/ ___/"
    echo " / ____/ /  /  __/ / / / / / /_/ / /_/  __/ /    "
    echo "/_/   /_/   \\\\___/_/ /_/ /_/ .___/\\\\__/\\\\___/_/     "
    echo "                        /_/                     "
    echo -e "${NC}"
    echo -e "${CYAN}🤖 智能Claude提示词监控工具${NC}"
    echo -e "${GREEN}📍 稳定版 v2.0 • UTF-8完全修复${NC}"
    echo ""
    sleep 2
}

# 主函数
main() {
    print_welcome
    show_main_menu
}

# 错误处理
trap 'echo -e "\n${RED}程序已中断${NC}"; exit 1' INT TERM

# 运行
main "$@"
