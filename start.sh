#!/bin/bash

# Prompter AI反馈系统 - 一键启动脚本
# 用户友好的开源配置和启动工具

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

clear

echo -e "${BLUE}"
echo "╔══════════════════════════════════════════════════════════╗"
echo "║          🚀 Prompter AI反馈系统 - 一键启动器            ║"
echo "║        智能提示词分析 + 优化建议 + Telegram推送          ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo -e "${NC}"
echo ""

echo -e "${GREEN}✨ 欢迎使用Prompter开源AI反馈系统！${NC}"
echo -e "${WHITE}   这是一个完全开源的项目，旨在帮助用户优化Claude提示词${NC}"
echo ""

echo -e "${CYAN}🎯 系统特色:${NC}"
echo -e "${WHITE}   • 🧠 Google Gemini思考模型: 深度分析您的提示词质量${NC}"
echo -e "${WHITE}   • ⚡ 智能降级机制: 配额不足自动切换快速模型${NC}"
echo -e "${WHITE}   • 📱 Telegram推送: 每日个性化优化建议${NC}"
echo -e "${WHITE}   • 🔄 多种监控模式: 历史/会话/Shell监控${NC}"
echo -e "${WHITE}   • 🔐 安全私有: 您的API密钥仅存储在本地${NC}"
echo ""

# 检查可执行文件
if [ ! -f "./target/release/prompter" ]; then
    echo -e "${RED}❌ 错误: 未找到可执行文件${NC}"
    echo -e "${YELLOW}💡 请确保已经编译项目: cargo build --release${NC}"
    echo ""
    read -p "按 Enter 键退出..." dummy
    exit 1
fi

echo -e "${PURPLE}🛠️  选择操作:${NC}"
echo ""
echo -e "${CYAN}1.${NC} 🚀 ${WHITE}一键配置系统${NC}     - 首次使用推荐（自动打开浏览器引导）"
echo -e "${CYAN}2.${NC} 📊 ${WHITE}启动历史监控${NC}     - 监控Claude Code历史记录"
echo -e "${CYAN}3.${NC} 🔍 ${WHITE}启动会话监控${NC}     - 实时监控Claude进程交互"
echo -e "${CYAN}4.${NC} 🤖 ${WHITE}启动AI分析服务${NC}   - 自动分析并推送优化建议"
echo -e "${CYAN}5.${NC} 🧪 ${WHITE}测试系统配置${NC}     - 检查API连接状态"
echo -e "${CYAN}6.${NC} 📖 ${WHITE}查看帮助文档${NC}     - 完整使用指南"
echo -e "${CYAN}7.${NC} 🎨 ${WHITE}演示系统功能${NC}     - 查看功能演示"
echo -e "${CYAN}0.${NC} ❌ ${WHITE}退出${NC}"
echo ""

while true; do
    read -p "请选择操作 (0-7): " choice
    echo ""

    case $choice in
        1)
            echo -e "${GREEN}🚀 启动一键配置向导...${NC}"
            echo -e "${YELLOW}💡 系统将自动打开浏览器帮助您获取API密钥${NC}"
            echo ""
            ./target/release/prompter --setup-feedback
            break
            ;;
        2)
            echo -e "${GREEN}📊 启动历史监控模式...${NC}"
            echo -e "${YELLOW}📝 将监控 ~/.claude/history.jsonl 文件变化${NC}"
            echo -e "${YELLOW}⏹️  按 Ctrl+C 停止监控${NC}"
            echo ""
            ./target/release/prompter --history-monitor
            break
            ;;
        3)
            echo -e "${GREEN}🔍 启动会话监控模式...${NC}"
            echo -e "${YELLOW}🎯 将监控活跃的Claude Code进程交互${NC}"
            echo -e "${YELLOW}⏹️  按 Ctrl+C 停止监控${NC}"
            echo ""
            ./target/release/prompter --session-monitor
            break
            ;;
        4)
            echo -e "${GREEN}🤖 启动AI分析服务...${NC}"
            echo -e "${YELLOW}📱 将定期分析提示词并推送到Telegram${NC}"
            echo -e "${YELLOW}⏹️  按 Ctrl+C 停止服务${NC}"
            echo ""
            ./target/release/prompter --auto-feedback
            break
            ;;
        5)
            echo -e "${GREEN}🧪 测试系统配置...${NC}"
            echo ""
            ./target/release/prompter --test-feedback
            echo ""
            echo -e "${CYAN}按 Enter 键返回主菜单...${NC}"
            read dummy
            exec "$0"
            ;;
        6)
            echo -e "${GREEN}📖 显示帮助文档...${NC}"
            echo ""
            ./target/release/prompter --help-feedback
            echo ""
            echo -e "${CYAN}按 Enter 键返回主菜单...${NC}"
            read dummy
            exec "$0"
            ;;
        7)
            echo -e "${GREEN}🎨 显示功能演示...${NC}"
            echo ""
            if [ -f "./demo_ai_feedback.sh" ]; then
                ./demo_ai_feedback.sh
            else
                echo -e "${RED}❌ 未找到演示脚本文件${NC}"
            fi
            echo ""
            echo -e "${CYAN}按 Enter 键返回主菜单...${NC}"
            read dummy
            exec "$0"
            ;;
        0)
            echo -e "${GREEN}👋 感谢使用Prompter AI反馈系统！${NC}"
            echo -e "${CYAN}🌟 如果觉得有用，请给我们一个GitHub Star！${NC}"
            echo -e "${YELLOW}🐛 遇到问题？欢迎提交Issue反馈${NC}"
            echo ""
            exit 0
            ;;
        *)
            echo -e "${RED}❌ 无效选择，请输入 0-7${NC}"
            echo ""
            ;;
    esac
done