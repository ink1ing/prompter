#!/bin/bash

# Prompter 一键启动脚本 - 智能检测和启动

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 输出带颜色的文本
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
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

# 检查依赖
check_dependencies() {
    print_status "检查系统依赖..."

    # 检查Rust
    if ! command -v cargo &> /dev/null; then
        print_error "未找到Rust/Cargo，请先安装Rust"
        exit 1
    fi

    # 检查Claude CLI
    if ! command -v claude &> /dev/null; then
        print_warning "未找到Claude CLI，部分功能可能不可用"
    fi

    print_success "依赖检查完成"
}

# 智能编译
smart_build() {
    print_status "检查编译状态..."

    if [ ! -f "./target/release/prompter" ]; then
        print_status "首次运行，开始编译..."
        cargo build --release
        print_success "编译完成"
    else
        # 检查源码是否有更新
        if [ "src/" -nt "./target/release/prompter" ] || [ "Cargo.toml" -nt "./target/release/prompter" ]; then
            print_status "检测到源码更新，重新编译..."
            cargo build --release
            print_success "重新编译完成"
        else
            print_success "二进制文件已是最新"
        fi
    fi
}

# 智能启动选择
smart_launch() {
    print_status "Prompter 智能启动向导"
    echo ""
    echo "🎯 选择启动模式："
    echo "1) 快速启动（简化模式）"
    echo "2) 完整监控（PTY模式）"
    echo "3) 性能测试模式"
    echo "4) 后台服务模式"
    echo "5) 自定义模式"
    echo ""

    # 自动检测最佳模式
    if [ "$1" = "auto" ]; then
        print_status "自动模式：选择简化模式启动"
        mode=1
    else
        read -p "请选择 (1-5) [默认: 1]: " mode
        mode=${mode:-1}
    fi

    case $mode in
        1)
            print_status "启动简化模式..."
            ./target/release/prompter --simple
            ;;
        2)
            print_status "启动PTY监控模式..."
            ./target/release/prompter
            ;;
        3)
            print_status "运行性能测试..."
            ./target/release/prompter --benchmark
            ;;
        4)
            print_status "启动后台服务模式..."
            nohup ./target/release/prompter --simple > prompter.log 2>&1 &
            echo $! > prompter.pid
            print_success "后台服务已启动，PID: $(cat prompter.pid)"
            print_status "查看日志: tail -f prompter.log"
            print_status "停止服务: kill $(cat prompter.pid)"
            ;;
        5)
            read -p "输入自定义参数: " custom_params
            print_status "启动自定义模式..."
            ./target/release/prompter $custom_params
            ;;
        *)
            print_error "无效选择"
            exit 1
            ;;
    esac
}

# 主函数
main() {
    echo "🚀 Prompter 智能启动器"
    echo "======================"
    echo ""

    check_dependencies
    smart_build
    smart_launch "$1"
}

# 脚本入口
if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    echo "用法: $0 [auto]"
    echo ""
    echo "参数:"
    echo "  auto    自动选择最佳启动模式"
    echo "  --help  显示此帮助信息"
    exit 0
fi

main "$1"