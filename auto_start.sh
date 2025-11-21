#!/bin/bash

# Prompter 自动化启动脚本

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
    echo "🤖 Prompter 自动化配置向导"
    echo "=============================="
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

# 配置文件设置
setup_config() {
    print_step "3" "配置文件设置..."

    if [ -f "config.toml" ]; then
        echo "发现现有配置文件"
        read -p "是否重新配置? (y/n) [n]: " reconfigure
        if [ "$reconfigure" != "y" ]; then
            return
        fi
    fi

    echo ""
    echo "请输入 Cloudflare 配置信息："
    echo "（可在 Cloudflare 仪表板中找到这些信息）"
    echo ""

    # Cloudflare Account ID
    read -p "Account ID: " cf_account_id
    while [ -z "$cf_account_id" ]; do
        print_warning "Account ID 不能为空"
        read -p "Account ID: " cf_account_id
    done

    # Zone ID
    read -p "Zone ID (域名的Zone ID): " cf_zone_id
    while [ -z "$cf_zone_id" ]; do
        print_warning "Zone ID 不能为空"
        read -p "Zone ID: " cf_zone_id
    done

    # API Token
    read -p "API Token (需要Zone:Edit权限): " cf_api_token
    while [ -z "$cf_api_token" ]; do
        print_warning "API Token 不能为空"
        read -p "API Token: " cf_api_token
    done

    # 域名
    read -p "网站域名 (如: example.com): " domain
    while [ -z "$domain" ]; do
        print_warning "域名不能为空"
        read -p "域名: " domain
    done

    # GitHub 仓库
    read -p "GitHub仓库 (如: username/repo): " github_repo
    while [ -z "$github_repo" ]; do
        print_warning "GitHub仓库不能为空"
        read -p "GitHub仓库: " github_repo
    done

    # 上传间隔
    read -p "自动上传间隔（小时） [1]: " upload_interval
    upload_interval=${upload_interval:-1}

    # 生成配置文件
    cat > config.toml << EOF
# Prompter 自动化配置
[app]
name = "prompter"
version = "1.0.0"
auto_upload_enabled = true
upload_interval_hours = $upload_interval

[filter]
# 中文检测设置
detect_chinese = true
min_chinese_chars = 3
exclude_commands = ["/", "quit", "exit", "help"]

[cloudflare]
# Cloudflare 配置
account_id = "$cf_account_id"
zone_id = "$cf_zone_id"
api_token = "$cf_api_token"

[website]
# 网站配置
domain = "$domain"
upload_endpoint = "/api/prompts"
github_repo = "$github_repo"

[storage]
# 本地存储配置
data_dir = "./data"
prompts_file = "prompts.md"
backup_dir = "./backups"
max_backups = 10

[logging]
level = "info"
file = "prompter.log"
max_size_mb = 50
EOF

    print_success "配置文件已生成"
}

# 创建Cloudflare Pages Function
create_pages_function() {
    print_step "4" "生成 Cloudflare Pages Function 代码..."

    mkdir -p functions/api

    cat > functions/api/prompts.js << 'EOF'
// Cloudflare Pages Function - 接收提示词上传
export async function onRequestPost(context) {
  try {
    // 验证请求
    const authHeader = context.request.headers.get('Authorization');
    if (!authHeader || !authHeader.startsWith('Bearer ')) {
      return new Response('Unauthorized', { status: 401 });
    }

    const batch = await context.request.json();

    // 验证数据格式
    if (!batch.prompts || !Array.isArray(batch.prompts)) {
      return new Response('Invalid batch format', { status: 400 });
    }

    // 存储到KV
    const key = `prompts:${batch.id}`;
    await context.env.PROMPTER_KV.put(key, JSON.stringify(batch), {
      expirationTtl: 30 * 24 * 60 * 60 // 30天过期
    });

    // 更新索引
    const indexKey = 'prompt_index';
    const existingIndex = await context.env.PROMPTER_KV.get(indexKey, 'json') || [];
    existingIndex.push({
      id: batch.id,
      timestamp: batch.timestamp,
      count: batch.total_count,
      chinese_chars: batch.chinese_char_count
    });

    // 保持最近100个批次
    const recentIndex = existingIndex.slice(-100);
    await context.env.PROMPTER_KV.put(indexKey, JSON.stringify(recentIndex));

    console.log(`Stored batch ${batch.id} with ${batch.total_count} prompts`);

    return new Response(JSON.stringify({
      success: true,
      batch_id: batch.id,
      stored_prompts: batch.total_count,
      timestamp: new Date().toISOString()
    }), {
      headers: {
        'Content-Type': 'application/json',
        'Access-Control-Allow-Origin': '*'
      }
    });

  } catch (error) {
    console.error('Error processing batch:', error);
    return new Response(`Error: ${error.message}`, { status: 500 });
  }
}

// 处理OPTIONS请求（CORS预检）
export async function onRequestOptions(context) {
  return new Response(null, {
    status: 200,
    headers: {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'POST, OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type, Authorization',
    },
  });
}
EOF

    print_success "Pages Function 代码已生成: functions/api/prompts.js"
}

# 测试配置
test_configuration() {
    print_step "5" "测试配置..."

    echo "正在测试 Cloudflare API 连接..."
    ./target/release/prompter --upload --config config.toml > /dev/null 2>&1

    if [ $? -eq 0 ]; then
        print_success "配置测试通过"
    else
        print_warning "配置测试失败，请检查配置文件"
        echo "可以手动运行以下命令进行详细测试:"
        echo "./target/release/prompter --upload --config config.toml"
    fi
}

# 启动选项
show_start_options() {
    print_step "6" "启动选项..."

    echo ""
    echo "🚀 现在你可以使用以下方式启动 Prompter："
    echo ""
    echo "1️⃣  自动化模式 (每${upload_interval}小时自动上传):"
    echo "   ./target/release/prompter --auto"
    echo ""
    echo "2️⃣  手动上传模式:"
    echo "   ./target/release/prompter --upload"
    echo ""
    echo "3️⃣  简化监控模式:"
    echo "   ./target/release/prompter --simple"
    echo ""
    echo "4️⃣  后台服务模式:"
    echo "   nohup ./target/release/prompter --auto > prompter.log 2>&1 &"
    echo ""

    read -p "现在启动自动化模式? (y/n) [y]: " start_now
    start_now=${start_now:-y}

    if [ "$start_now" = "y" ]; then
        echo ""
        print_success "启动自动化模式..."
        echo "按 Ctrl+C 停止服务"
        echo ""
        ./target/release/prompter --auto
    fi
}

# 主函数
main() {
    print_header

    check_and_build
    setup_config
    create_pages_function
    test_configuration
    show_start_options

    echo ""
    print_success "自动化配置完成！"
    echo ""
    echo "📋 接下来的步骤："
    echo "1. 将 functions/ 目录上传到你的 Cloudflare Pages 项目"
    echo "2. 在 Cloudflare Pages 中设置 KV 命名空间绑定: PROMPTER_KV"
    echo "3. 运行 ./target/release/prompter --auto 启动自动化服务"
}

# 错误处理
trap 'print_error "脚本执行中断"; exit 1' INT TERM

# 运行主函数
main "$@"