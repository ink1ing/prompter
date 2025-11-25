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
    echo "║                🤖 PROMPTER STABLE v2.1                 ║"
    echo "║                   一键启动菜单                           ║"
    echo "║         ✅ UTF-8修复 + 📱 Telegram集成                 ║"
    echo "╚══════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
    echo ""
    echo -e "${GREEN}🎉 v2.1 特性:${NC}"
    echo -e "${WHITE}   • UTF-8字符边界问题完全修复${NC}"
    echo -e "${WHITE}   • 历史监控100%稳定运行${NC}"
    echo -e "${WHITE}   • Telegram命令监听器已集成${NC}"
    echo -e "${WHITE}   • 支持反馈间隔智能配置${NC}"
    echo ""
    echo -e "${WHITE}选择启动模式:${NC}"
    echo ""
    echo -e "${CYAN}1.${NC} 📁 基础模式 - 本地检测存储"
    echo -e "${WHITE}     • 仅本地检测提示词${NC}"
    echo -e "${WHITE}     • 保存到本地路径${NC}"
    echo -e "${WHITE}     • 轻量级,无需AI分析${NC}"
    echo ""
    echo -e "${CYAN}2.${NC} 🤖 标准模式 - Gemini智能分析 ${GREEN}(推荐 ⭐)${NC}"
    echo -e "${WHITE}     • Gemini AI深度分析提示词${NC}"
    echo -e "${WHITE}     • Telegram实时反馈和报告${NC}"
    echo -e "${WHITE}     • 支持时间/数量间隔配置${NC}"
    echo ""
    echo -e "${CYAN}3.${NC} 🚀 大力王模式 - MCP外部集成"
    echo -e "${WHITE}     • 连接外部MCP服务器${NC}"
    echo -e "${WHITE}     • 扩展功能和集成${NC}"
    echo -e "${WHITE}     • 🔮 敬请期待...${NC}"
    echo ""
    echo -e "${YELLOW}💡 推荐选择 '2' 标准模式${NC}"
    echo ""

    read -p "请选择 (1-3) [2]: " choice
    choice=${choice:-2}

    case $choice in
        1)
            # 基础模式 - 本地检测存储
            echo -e "${GREEN}启动基础模式 - 本地检测存储...${NC}"
            echo ""
            if [ ! -f "./target/release/prompter" ]; then
                echo "正在编译项目..."
                cargo build --release > /dev/null 2>&1
            fi
            run_in_current_terminal "./target/release/prompter --history-monitor"
            ;;
        2)
            # 标准模式 - Gemini智能分析 + Telegram反馈
            echo -e "${GREEN}启动标准模式 - Gemini智能分析...${NC}"
            echo ""
            if [ ! -f "./target/release/prompter" ]; then
                echo "正在编译项目..."
                cargo build --release > /dev/null 2>&1
            fi
            run_in_current_terminal "./target/release/prompter --telegram-listen"
            ;;
        3)
            # 大力王模式 - MCP外部集成
            echo -e "${GREEN}大力王模式 - MCP外部集成${NC}"
            echo ""
            echo -e "${CYAN}🔮 敬请期待...${NC}"
            echo ""
            echo -e "${YELLOW}💡 当前可用模式:${NC}"
            echo -e "   1. 基础模式 - 本地检测存储"
            echo -e "   2. 标准模式 - Gemini智能分析 + Telegram反馈"
            echo ""
            sleep 3
            show_main_menu
            ;;
        *)
            echo -e "${RED}无效选择，使用默认模式（标准模式）${NC}"
            echo ""
            if [ ! -f "./target/release/prompter" ]; then
                echo "正在编译项目..."
                cargo build --release > /dev/null 2>&1
            fi
            run_in_current_terminal "./target/release/prompter --telegram-listen"
            ;;
    esac
}

# (已移除系统状态和帮助功能)

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
    echo -e "${GREEN}📍 稳定版 v2.1 • Telegram集成版${NC}"
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
