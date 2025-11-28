#!/bin/bash

# 设置脚本目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# 设置颜色
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

clear
echo -e "${CYAN}================================"
echo -e "🎧 Prompter 智能监听器 v2.2"
echo -e "================================${NC}"
echo ""
echo -e "${YELLOW}请选择启动模式：${NC}"
echo ""
echo -e "${GREEN}1. 📁 基础监测模式${NC}"
echo "      • 仅监测和保存到本地文件"
echo "      • 无需API密钥"
echo "      • 轻量级运行"
echo ""
echo -e "${BLUE}2. 🎧 TG Bot + LLM集成模式 [推荐]${NC}"
echo "      • Telegram Bot智能监听器"
echo "      • 完整LLM配置流程 (Gemini/Perplexity)"
echo "      • 自动AI分析和反馈"
echo "      • 需要配置API密钥"
echo ""
echo -e "${PURPLE}3. 🔌 MCP模式 [开发中]${NC}"
echo "      • Model Context Protocol集成"
echo "      • 高级AI工作流"
echo "      • 暂未实现"
echo ""

read -p "请输入选择 (1-3): " choice

case $choice in
    1)
        echo -e "\n${GREEN}启动 基础监测模式...${NC}"
        echo "• 选择监测类型："
        echo "  1) Shell命令监听器"
        echo "  2) Claude Code会话监控"
        echo ""
        read -p "请选择 (1-2): " monitor_choice

        case $monitor_choice in
            1)
                echo -e "\n${GREEN}启动 Shell 命令监听器...${NC}"
                read -p "按回车键继续..."
                cd "$PROJECT_DIR"
                if command -v cargo >/dev/null 2>&1; then
                    cargo run --release -- --shell-monitor
                else
                    echo -e "${RED}错误：未找到 Rust/Cargo，请先安装 Rust 开发环境${NC}"
                    exit 1
                fi
                ;;
            2)
                echo -e "\n${GREEN}启动 Claude Code 会话监控...${NC}"
                read -p "按回车键继续..."
                cd "$PROJECT_DIR"
                if command -v cargo >/dev/null 2>&1; then
                    cargo run --release -- --session-monitor
                else
                    echo -e "${RED}错误：未找到 Rust/Cargo，请先安装 Rust 开发环境${NC}"
                    exit 1
                fi
                ;;
            *)
                echo -e "${RED}无效选择${NC}"
                exit 1
                ;;
        esac
        ;;

    2)
        echo -e "\n${GREEN}启动 TG Bot + LLM集成模式...${NC}"
        echo ""

        # 首先检查现有的API密钥配置
        echo -e "${YELLOW}🔑 检查现有API密钥配置...${NC}"
        cd "$PROJECT_DIR"

        # 读取现有配置
        CURRENT_GEMINI=""
        CURRENT_PERPLEXITY=""
        CONFIG_FILE="$PROJECT_DIR/config.toml"

        if [ -f "$CONFIG_FILE" ]; then
            CURRENT_GEMINI=$(grep '^gemini_api_key' "$CONFIG_FILE" | sed 's/.*= *"\([^"]*\)".*/\1/')
            CURRENT_PERPLEXITY=$(grep '^perplexity_api_key' "$CONFIG_FILE" | sed 's/.*= *"\([^"]*\)".*/\1/')
        fi

        # 显示当前状态
        echo ""
        echo "当前API密钥状态："
        if [ -n "$CURRENT_GEMINI" ] && [ "$CURRENT_GEMINI" != "" ]; then
            echo "  ✅ Google Gemini: 已配置 (${CURRENT_GEMINI:0:10}****)"
            HAS_GEMINI=true
        else
            echo "  📝 Google Gemini: 未配置"
            HAS_GEMINI=false
        fi

        if [ -n "$CURRENT_PERPLEXITY" ] && [ "$CURRENT_PERPLEXITY" != "" ]; then
            echo "  ✅ Perplexity AI: 已配置 (${CURRENT_PERPLEXITY:0:10}****)"
            HAS_PERPLEXITY=true
        else
            echo "  📝 Perplexity AI: 未配置"
            HAS_PERPLEXITY=false
        fi

        # 配置Gemini API密钥
        echo ""
        echo -e "${YELLOW}🔧 配置 Google Gemini API密钥${NC}"
        if [ "$HAS_GEMINI" = true ]; then
            echo "当前已配置Gemini API密钥"
            read -p "是否要更新Gemini API密钥？(y/N): " update_gemini
            if [[ "$update_gemini" =~ ^[Yy]$ ]]; then
                read -p "请输入新的Gemini API Key (以AIzaSy开头): " NEW_GEMINI_KEY
                if [ -n "$NEW_GEMINI_KEY" ]; then
                    GEMINI_API_KEY="$NEW_GEMINI_KEY"
                    echo -e "${GREEN}✅ Gemini API密钥已更新${NC}"
                else
                    GEMINI_API_KEY="$CURRENT_GEMINI"
                    echo -e "${YELLOW}保持原有Gemini API密钥${NC}"
                fi
            else
                GEMINI_API_KEY="$CURRENT_GEMINI"
                echo -e "${YELLOW}保持原有Gemini API密钥${NC}"
            fi
        else
            read -p "请输入Gemini API Key (可跳过，按回车): " NEW_GEMINI_KEY
            if [ -n "$NEW_GEMINI_KEY" ]; then
                # 验证Gemini API密钥格式
                if [[ "$NEW_GEMINI_KEY" =~ ^AIzaSy.* ]] && [ ${#NEW_GEMINI_KEY} -gt 30 ]; then
                    GEMINI_API_KEY="$NEW_GEMINI_KEY"
                    echo -e "${GREEN}✅ Gemini API密钥格式正确${NC}"
                else
                    echo -e "${RED}⚠️ Gemini API密钥格式可能不正确（应以AIzaSy开头），但仍将保存${NC}"
                    GEMINI_API_KEY="$NEW_GEMINI_KEY"
                fi
            else
                GEMINI_API_KEY=""
                echo -e "${YELLOW}跳过Gemini API密钥配置${NC}"
            fi
        fi

        # 配置Perplexity API密钥
        echo ""
        echo -e "${YELLOW}🔧 配置 Perplexity AI API密钥${NC}"
        if [ "$HAS_PERPLEXITY" = true ]; then
            echo "当前已配置Perplexity API密钥"
            read -p "是否要更新Perplexity API密钥？(y/N): " update_perplexity
            if [[ "$update_perplexity" =~ ^[Yy]$ ]]; then
                read -p "请输入新的Perplexity API Key (以pplx-开头): " NEW_PERPLEXITY_KEY
                if [ -n "$NEW_PERPLEXITY_KEY" ]; then
                    PERPLEXITY_API_KEY="$NEW_PERPLEXITY_KEY"
                    echo -e "${GREEN}✅ Perplexity API密钥已更新${NC}"
                else
                    PERPLEXITY_API_KEY="$CURRENT_PERPLEXITY"
                    echo -e "${YELLOW}保持原有Perplexity API密钥${NC}"
                fi
            else
                PERPLEXITY_API_KEY="$CURRENT_PERPLEXITY"
                echo -e "${YELLOW}保持原有Perplexity API密钥${NC}"
            fi
        else
            read -p "请输入Perplexity API Key (可跳过，按回车): " NEW_PERPLEXITY_KEY
            if [ -n "$NEW_PERPLEXITY_KEY" ]; then
                # 验证Perplexity API密钥格式
                if [[ "$NEW_PERPLEXITY_KEY" =~ ^pplx-.* ]] && [ ${#NEW_PERPLEXITY_KEY} -gt 40 ]; then
                    PERPLEXITY_API_KEY="$NEW_PERPLEXITY_KEY"
                    echo -e "${GREEN}✅ Perplexity API密钥格式正确${NC}"
                else
                    echo -e "${RED}⚠️ Perplexity API密钥格式可能不正确（应以pplx-开头），但仍将保存${NC}"
                    PERPLEXITY_API_KEY="$NEW_PERPLEXITY_KEY"
                fi
            else
                PERPLEXITY_API_KEY=""
                echo -e "${YELLOW}跳过Perplexity API密钥配置${NC}"
            fi
        fi

        # 检查是否至少有一个API密钥
        if [ -z "$GEMINI_API_KEY" ] && [ -z "$PERPLEXITY_API_KEY" ]; then
            echo -e "\n${RED}❌ 错误：至少需要配置一个API密钥${NC}"
            echo "请重新运行脚本并配置Gemini或Perplexity API密钥"
            exit 1
        fi

        # 显示最终配置状态
        echo ""
        echo -e "${CYAN}📋 API密钥配置完成：${NC}"
        if [ -n "$GEMINI_API_KEY" ]; then
            echo "  🟢 Gemini: ${GEMINI_API_KEY:0:10}****"
        else
            echo "  ⚪ Gemini: 未配置"
        fi
        if [ -n "$PERPLEXITY_API_KEY" ]; then
            echo "  🟢 Perplexity: ${PERPLEXITY_API_KEY:0:10}****"
        else
            echo "  ⚪ Perplexity: 未配置"
        fi

        # 选择主要使用的LLM提供商
        echo ""
        echo -e "${YELLOW}选择主要使用的 LLM 提供商：${NC}"

        # 根据可用的API密钥生成选项
        PROVIDERS=()
        if [ -n "$GEMINI_API_KEY" ]; then
            PROVIDERS+=("gemini")
            echo "1. Google Gemini"
        fi
        if [ -n "$PERPLEXITY_API_KEY" ]; then
            if [ ${#PROVIDERS[@]} -eq 0 ]; then
                PROVIDERS+=("perplexity")
                echo "1. Perplexity AI"
            else
                PROVIDERS+=("perplexity")
                echo "2. Perplexity AI"
            fi
        fi

        echo ""
        read -p "请选择 (1-${#PROVIDERS[@]}): " provider_choice

        # 验证选择并设置主要提供商
        if [ "$provider_choice" -ge 1 ] && [ "$provider_choice" -le ${#PROVIDERS[@]} ]; then
            LLM_PROVIDER="${PROVIDERS[$((provider_choice-1))]}"
            case "$LLM_PROVIDER" in
                "gemini")
                    echo -e "\n${GREEN}已选择：Google Gemini 作为主要提供商${NC}"
                    ;;
                "perplexity")
                    echo -e "\n${GREEN}已选择：Perplexity AI 作为主要提供商${NC}"
                    ;;
            esac
        else
            echo -e "\n${RED}无效选择，默认使用第一个可用提供商${NC}"
            LLM_PROVIDER="${PROVIDERS[0]}"
        fi

        # 模式选择
        echo ""
        echo -e "${YELLOW}选择运行模式：${NC}"
        echo "1. 🚀 快速模式 (推荐)"
        echo "2. 🧠 高级分析模式"
        echo ""
        read -p "请选择 (1-2): " mode_choice

        case $mode_choice in
            1)
                LLM_MODE="fast"
                echo -e "\n${GREEN}已选择：快速模式${NC}"
                ;;
            2)
                LLM_MODE="thinking"
                echo -e "\n${GREEN}已选择：高级分析模式${NC}"
                ;;
            *)
                LLM_MODE="fast"
                echo -e "\n${YELLOW}默认选择：快速模式${NC}"
                ;;
        esac

        # 更新配置文件
        echo ""
        echo -e "${YELLOW}正在更新配置文件...${NC}"

        CONFIG_FILE="$PROJECT_DIR/config.toml"

        # 备份原配置
        cp "$CONFIG_FILE" "$CONFIG_FILE.backup.$(date +%Y%m%d_%H%M%S)" 2>/dev/null

        # 读取现有配置并更新LLM相关设置
        if [ -f "$CONFIG_FILE" ]; then
            # 使用临时文件更新配置
            TEMP_CONFIG=$(mktemp)

            # 读取配置文件并更新相关字段
            awk -v llm_provider="$LLM_PROVIDER" \
                -v llm_mode="$LLM_MODE" \
                -v gemini_key="$GEMINI_API_KEY" \
                -v perplexity_key="$PERPLEXITY_API_KEY" '
                /^\[ai_feedback\]/ { in_ai_section = 1 }
                /^\[/ && !/^\[ai_feedback\]/ { in_ai_section = 0 }

                in_ai_section && /^llm_provider/ {
                    print "llm_provider = \"" llm_provider "\""
                    next
                }
                in_ai_section && /^llm_mode/ {
                    print "llm_mode = \"" llm_mode "\""
                    next
                }
                in_ai_section && /^gemini_api_key/ {
                    print "gemini_api_key = \"" gemini_key "\""
                    next
                }
                in_ai_section && /^perplexity_api_key/ {
                    print "perplexity_api_key = \"" perplexity_key "\""
                    next
                }
                { print }
            ' "$CONFIG_FILE" > "$TEMP_CONFIG"

            mv "$TEMP_CONFIG" "$CONFIG_FILE"
        else
            echo -e "${RED}错误：配置文件不存在${NC}"
            exit 1
        fi

        echo -e "${GREEN}配置已更新！${NC}"
        echo ""
        echo -e "${CYAN}配置摘要：${NC}"
        echo "• LLM提供商: $LLM_PROVIDER"
        echo "• 运行模式: $LLM_MODE"
        echo "• API Key: ****已配置****"
        echo ""

        read -p "按回车键开始启动程序..."

        cd "$PROJECT_DIR"
        if command -v cargo >/dev/null 2>&1; then
            cargo run --release -- --telegram-listen
        else
            echo -e "${RED}错误：未找到 Rust/Cargo，请先安装 Rust 开发环境${NC}"
            exit 1
        fi
        ;;

    3)
        echo -e "\n${PURPLE}MCP模式暂未实现${NC}"
        echo "该模式正在开发中，敬请期待..."
        echo ""
        echo "目前可用的模式："
        echo "• 基础监测模式 (选项1)"
        echo "• TG Bot + LLM集成模式 (选项2)"
        exit 0
        ;;

    *)
        echo -e "${RED}无效选择，程序退出${NC}"
        exit 1
        ;;
esac