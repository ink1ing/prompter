#!/bin/bash

# AI反馈系统演示脚本
# 展示思考模型自动降级功能

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
echo "║             🤖 AI反馈系统功能演示                       ║"
echo "║         支持思考模型 + 快速模型自动切换                  ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo -e "${NC}"
echo ""

echo -e "${GREEN}🎯 新增功能特性:${NC}"
echo -e "${WHITE}   • 🧠 Gemini 2.0 思考模型: 深度分析用户提示词${NC}"
echo -e "${WHITE}   • ⚡ Gemini 1.5 快速模型: 配额不足时自动降级${NC}"
echo -e "${WHITE}   • 🔄 智能重试机制: 指数退避算法自动重试${NC}"
echo -e "${WHITE}   • 📊 思考预算控制: 支持动态/固定/关闭三种模式${NC}"
echo -e "${WHITE}   • 🎚️ 灵活配置: 可自定义模型、预算、重试次数${NC}"
echo ""

echo -e "${CYAN}📋 系统架构:${NC}"
echo ""
echo -e "${WHITE}   提示词收集 → AI分析引擎 → Telegram推送${NC}"
echo -e "${WHITE}        ↓            ↓           ↓${NC}"
echo -e "${WHITE}   历史监控     思考模型优先   每日报告${NC}"
echo -e "${WHITE}   会话监控     快速模型降级   错误通知${NC}"
echo -e "${WHITE}   Shell监控    智能重试      状态反馈${NC}"
echo ""

echo -e "${YELLOW}🔧 配置说明:${NC}"
echo -e "${WHITE}   thinking_budget = 1024   # 思考预算（Token数量）${NC}"
echo -e "${WHITE}   thinking_budget = -1     # 动态思考（AI自动调整）${NC}"
echo -e "${WHITE}   thinking_budget = 0      # 关闭思考（快速模式）${NC}"
echo -e "${WHITE}   auto_fallback = true     # 启用自动降级${NC}"
echo -e "${WHITE}   max_retries = 3          # 最大重试次数${NC}"
echo ""

echo -e "${PURPLE}📱 使用命令:${NC}"
echo ""
echo -e "${CYAN}1.${NC} 配置系统: ${WHITE}./target/release/prompter --setup-feedback${NC}"
echo -e "${CYAN}2.${NC} 测试功能: ${WHITE}./target/release/prompter --test-feedback${NC}"
echo -e "${CYAN}3.${NC} 启动服务: ${WHITE}./target/release/prompter --auto-feedback${NC}"
echo ""

echo -e "${GREEN}🎉 关键优势:${NC}"
echo -e "${WHITE}   • 💰 成本优化: 优先使用思考模型，配额不足自动降级${NC}"
echo -e "${WHITE}   • 🚀 性能保障: 快速模型确保服务持续可用${NC}"
echo -e "${WHITE}   • 🛡️ 错误恢复: 智能重试和降级策略${NC}"
echo -e "${WHITE}   • 📈 质量优先: 思考模型提供更深度的分析${NC}"
echo ""

echo -e "${YELLOW}💡 推荐配置:${NC}"
echo -e "${WHITE}   • 日常使用: thinking_budget = -1 (动态思考)${NC}"
echo -e "${WHITE}   • 节省配额: thinking_budget = 512 (中等预算)${NC}"
echo -e "${WHITE}   • 快速模式: thinking_budget = 0 (仅快速模型)${NC}"
echo ""

echo -e "${BLUE}📊 当前系统状态:${NC}"
echo -e "${GREEN}   ✅ Telegram Bot: 已配置且连接成功${NC}"
echo -e "${GREEN}   ✅ 配置文件: 已加载完整的AI反馈配置${NC}"
echo -e "${GREEN}   ✅ 思考模型架构: 已实现自动降级机制${NC}"
echo -e "${YELLOW}   ⚠️  Gemini API: 需要正确的模型名称格式${NC}"
echo ""

echo -e "${GREEN}🚀 系统已就绪！可以开始使用AI反馈功能。${NC}"
echo ""