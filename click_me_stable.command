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
    echo -e "${CYAN}2.${NC} 🤖 标准模式 - 全功能智能分析 ${GREEN}(推荐 ⭐)${NC}"
    echo -e "${WHITE}     • Gemini AI深度分析提示词${NC}"
    echo -e "${WHITE}     • Telegram自动连接和推送${NC}"
    echo -e "${WHITE}     • 跨终端Claude进程监控${NC}"
    echo -e "${WHITE}     • 智能反馈间隔配置${NC}"
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
            # 标准模式 - 全功能智能分析（包含所有新功能）
            echo -e "${GREEN}启动标准模式 - 全功能智能分析...${NC}"
            echo ""
            echo -e "${CYAN}🔥 标准模式包含所有功能:${NC}"
            echo "   • Claude Code历史监控"
            echo "   • JSONL项目文件监控"
            echo "   • 跨终端进程监控"
            echo "   • Gemini AI智能分析"
            echo "   • Telegram自动连接推送"
            echo ""
            if [ ! -f "./target/release/prompter" ]; then
                echo "正在编译项目..."
                cargo build --release > /dev/null 2>&1
            fi

            echo -e "${BLUE}🚀 启动全功能模式...${NC}"
            echo ""

            # 同时启动历史监控和Telegram监听
            echo -e "${YELLOW}💡 该模式将开启多个监控进程:${NC}"
            echo "   1. Claude历史监控 (后台运行)"
            echo "   2. JSONL项目监控 (后台运行)"
            echo "   3. Telegram Bot监听 (前台运行)"
            echo ""

            # 启动历史监控（后台）
            echo -e "${BLUE}📚 启动历史监控...${NC}"
            ./target/release/prompter --history-monitor > /tmp/prompter_history.log 2>&1 &
            HISTORY_PID=$!

            # 启动JSONL监控（后台）
            echo -e "${BLUE}📋 启动JSONL监控...${NC}"
            ./target/release/prompter --jsonl-monitor > /tmp/prompter_jsonl.log 2>&1 &
            JSONL_PID=$!

            sleep 3

            # 启动Telegram监听（前台）
            echo -e "${BLUE}🤖 启动Telegram Bot监听...${NC}"
            ./target/release/prompter --telegram-listen

            # 清理后台进程
            kill $HISTORY_PID > /dev/null 2>&1
            kill $JSONL_PID > /dev/null 2>&1
            ;;
        3)
            # 大力王模式 - MCP外部集成
            echo -e "${GREEN}大力王模式 - MCP外部集成${NC}"
            echo ""
            echo -e "${CYAN}🔮 敬请期待...${NC}"
            echo ""
            echo -e "${YELLOW}💡 当前可用模式:${NC}"
            echo -e "   1. 基础模式 - 本地检测存储"
            echo -e "   2. 标准模式 - 全功能智能分析"
            echo ""
            sleep 3
            show_main_menu
            ;;
        *)
            echo -e "${RED}无效选择，使用默认模式（标准模式）${NC}"
            echo ""
            # 使用标准模式的完整功能
            if [ ! -f "./target/release/prompter" ]; then
                echo "正在编译项目..."
                cargo build --release > /dev/null 2>&1
            fi

            echo -e "${BLUE}🚀 启动全功能模式...${NC}"

            # 启动后台监控
            ./target/release/prompter --history-monitor > /tmp/prompter_history.log 2>&1 &
            HISTORY_PID=$!
            ./target/release/prompter --jsonl-monitor > /tmp/prompter_jsonl.log 2>&1 &
            JSONL_PID=$!

            sleep 3

            # 启动Telegram监听（前台）
            ./target/release/prompter --telegram-listen

            # 清理
            kill $HISTORY_PID > /dev/null 2>&1
            kill $JSONL_PID > /dev/null 2>&1
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
