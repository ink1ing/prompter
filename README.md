# 🤖 Prompter

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)
[![GitHub](https://img.shields.io/badge/github-%23121011.svg?style=for-the-badge&logo=github&logoColor=white)](https://github.com/ink1ing/prompter)

**智能Claude提示词监控和收集工具**

Prompter 是一个用 Rust 编写的轻量级终端应用，专门用于监控和收集 Claude Code CLI 的中文提示词。它提供了独立终端监控、可视化界面和自动上传功能，让你轻松管理和分享优秀的提示词。

## ✨ 核心特性

### 🔍 独立终端监控
- **零影响体验**: 无需PTY包装，对Claude Code使用体验零影响
- **多Shell支持**: 支持bash、zsh、fish等主流Shell
- **实时检测**: 每5秒自动扫描Shell历史文件变化
- **智能过滤**: 自动识别和提取中文提示词

### 🎨 可视化界面
- **专业监控面板**: 类似HTTP状态码的响应系统
- **活跃终端检测**: 自动识别Terminal.app、iTerm.app等终端应用
- **实时状态跟踪**: 显示成功记录数、监控文件状态
- **毫秒级时间戳**: 精确的时间记录和日志

### ☁️ 自动化上传
- **Cloudflare集成**: 支持Pages Functions和KV存储
- **GitHub同步**: 自动推送到GitHub仓库
- **定时任务**: 可配置的小时级自动上传
- **多重备份**: 本地备份 + 云端存储双保险

## 🚀 快速开始

### 一键启动（推荐）

```bash
# 下载项目
git clone https://github.com/ink1ing/prompter.git
cd prompter

# 可视化启动向导
./start_visual_monitor.sh
```

### 手动启动

```bash
# 编译项目
cargo build --release

# 基础监控模式
./target/release/prompter --shell-monitor

# 监控 + 自动上传模式
./target/release/prompter --auto --shell-monitor
```

## 📊 界面预览

启动后会显示专业的监控界面：

```
╔══════════════════════════════════════════════════════════╗
║                    🤖 PROMPTER MONITOR                   ║
║               独立终端监控模式 - Claude提示词智能收集        ║
╚══════════════════════════════════════════════════════════╝
⏰ 启动时间: 2024-11-21 14:30:45
🔧 监控间隔: 5秒
────────────────────────────────────────────────────────────
📊 监控状态总览
────────────────────────────────────────────────────────────
🖥️  活跃终端 (2 个):
   ✅ Terminal.app
   ✅ iTerm.app

📂 监控文件 (2 个):
   ✅ ~/.zsh_history (25610 bytes)
   ✅ ~/.bash_history (75081 bytes)

🎯 目标命令模式:
   📝 claude
   📝 claude-code
   📝 claude-cli
```

检测到中文提示词时的HTTP风格响应：

```
📥 [14:30:45.123] 200 OK - 中文提示词已保存 | Records: 1
   └─ 📄 Content: 帮我写一个计算斐波那契数列的Python函数...

📥 [14:30:47.456] 200 OK - 中文提示词已保存 | Records: 2
   └─ 📄 Content: 优化这段代码的性能，让它运行得更快...
```

## 🛠️ 安装与配置

### 系统要求

- **操作系统**: macOS, Linux, Windows
- **Rust版本**: 1.70+
- **Shell**: bash, zsh, fish (任意一种)

### 依赖安装

```bash
# macOS (使用 Homebrew)
brew install rust

# Ubuntu/Debian
apt-get install rustc cargo

# 其他系统请参考 https://rustup.rs/
```

### 配置文件

如需自动上传功能，创建 `config.toml`：

```toml
[app]
auto_upload_enabled = true
upload_interval_hours = 1

[filter]
detect_chinese = true
min_chinese_chars = 3
exclude_commands = ["/", "quit", "exit", "help"]

[cloudflare]
account_id = "your-account-id"
zone_id = "your-zone-id"
api_token = "your-api-token"

[website]
domain = "your-domain.com"
upload_endpoint = "/api/prompts"
github_repo = "username/repo-name"

[storage]
data_dir = "./data"
prompts_file = "prompts.md"
backup_dir = "./backups"
max_backups = 10
```

## 📋 使用方法

### 基础监控

```bash
# 启动独立Shell监控
./target/release/prompter --shell-monitor

# 在其他终端运行Claude命令
claude "帮我写一个快速排序算法"
claude "优化这段JavaScript代码的性能"
```

提示词会自动保存到 `./data/shell_captured_prompts.md`

### 自动上传模式

```bash
# 启动监控 + 自动上传
./target/release/prompter --auto --shell-monitor
```

每小时自动上传中文提示词到Cloudflare，并推送到GitHub仓库。

### 其他模式

```bash
# 查看所有选项
./target/release/prompter --help

# 手动上传
./target/release/prompter --upload

# 简化监控模式
./target/release/prompter --simple

# 性能基准测试
./target/release/prompter --benchmark
```

## 🔧 启动脚本

项目提供多种启动方式：

| 脚本 | 功能 | 推荐场景 |
|------|------|----------|
| `start_visual_monitor.sh` | 可视化启动向导 | 首次使用/演示 |
| `auto_start.sh` | 自动化配置向导 | 需要配置上传功能 |
| `start_shell_monitor.sh` | Shell监控配置 | 基础监控使用 |
| `demo_visual.sh` | 功能演示预览 | 了解功能特性 |

## 📁 项目结构

```
prompter/
├── src/                    # 源代码
│   ├── main.rs            # 主程序入口
│   ├── shell_monitor.rs   # Shell监控模块
│   ├── chinese_filter.rs  # 中文过滤器
│   ├── cloudflare_uploader.rs # Cloudflare上传
│   ├── auto_scheduler.rs  # 自动化调度器
│   └── ...
├── data/                  # 数据目录
│   └── shell_captured_prompts.md
├── backups/              # 备份目录
├── functions/            # Cloudflare Functions
│   └── api/prompts.js
├── *.sh                  # 启动脚本
├── config.toml           # 配置文件
└── README.md
```

## 🌐 Cloudflare 集成

Prompter 支持两种上传方式：

### 1. Pages Functions（推荐）

将生成的 `functions/api/prompts.js` 上传到你的 Cloudflare Pages 项目。

### 2. KV 存储

在 Cloudflare 仪表板中：
1. 创建 KV 命名空间
2. 绑定到 Pages 项目，绑定名称: `PROMPTER_KV`

## 🤝 贡献

欢迎贡献代码、报告问题或提出建议！

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## 📝 更新日志

### v1.0.0 (2024-11-21)

- ✨ 初始版本发布
- 🔍 独立Shell监控功能
- 🎨 可视化监控界面
- 📊 HTTP状态码风格响应
- 🖥️ 活跃终端自动检测
- ☁️ Cloudflare自动上传
- 📱 GitHub集成推送
- 🐚 多Shell支持 (bash/zsh/fish)

## ❓ 常见问题

**Q: 为什么没有检测到历史文件？**
A: 确保你的Shell启用了历史记录，并且已经运行过一些Claude命令。

**Q: 如何查看详细日志？**
A: 运行 `./target/release/prompter --shell-monitor 2>&1 | tee monitor.log`

**Q: 支持哪些操作系统？**
A: 支持 macOS、Linux 和 Windows（通过WSL）。

## 📄 许可证

本项目采用 [MIT License](LICENSE) 开源协议。

## 🙏 致谢

- [Claude](https://claude.ai/) - 优秀的AI助手
- [Rust](https://www.rust-lang.org/) - 系统编程语言
- [Cloudflare](https://www.cloudflare.com/) - 边缘计算平台
- [Tokio](https://tokio.rs/) - 异步运行时

---

⭐ 如果这个项目对你有帮助，请给个 Star！

📧 问题反馈: [Issues](https://github.com/ink1ing/prompter/issues)

🔗 项目主页: [https://github.com/ink1ing/prompter](https://github.com/ink1ing/prompter)