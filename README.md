# 🤖 Prompter

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)
[![Stable: v2.0](https://img.shields.io/badge/stable-v2.0-green.svg?style=for-the-badge)](https://github.com/ink1ing/prompter)

**智能Claude提示词监控工具 - 稳定版v2.0**

Prompter 是一个用 Rust 编写的轻量级终端应用，专门用于监控和收集 Claude Code CLI 的中文提示词，帮助你轻松记录、复盘和分享优秀的提示词。

## ✨ 稳定版v2.0特性

### 🎯 完全修复
- ✅ **UTF-8字符边界问题完全解决** - 不再因中文字符截取而崩溃
- ✅ **历史监控100%稳定运行** - 经过全面测试和验证
- ✅ **三种监控模式全部可用** - Shell监控、会话监控、历史监控

### 🔍 三种监控模式

#### 1. 📚 历史监控模式 ⭐ 推荐
直接监控Claude Code内部历史文件，最精准的监控方式。

```bash
./target/release/prompter --history-monitor
```

**优势**：
- 🎯 **最精准**: 直接读取Claude Code官方历史记录
- ✅ **100%稳定**: 完全修复UTF-8问题，不再崩溃
- 📝 **完整捕获**: 捕获所有用户中文交互
- 🏗️ **项目上下文**: 记录每条提示词的项目路径

#### 2. 🎨 Shell监控模式
监控Shell历史文件（bash/zsh/fish），独立终端运行。

```bash
./target/release/prompter --shell-monitor
```

**特点**：
- 🚫 无PTY包装，零影响
- 🔄 实时检测Shell历史变化
- 🌐 支持多种Shell

#### 3. 🎯 会话监控模式
监控活跃的Claude Code进程和tmux会话。

```bash
./target/release/prompter --session-monitor
```

**特点**：
- 📺 tmux集成
- ⚡ 实时交互捕获
- 🔍 进程自动检测

## 🚀 快速开始

### 方法1: 一键启动（最简单）

双击运行 `click_me_stable.command` 文件，选择启动模式即可。

**推荐选择**: 选项3 - 📚 历史监控模式（最稳定最精准）

### 方法2: 命令行启动

```bash
# 克隆项目
git clone https://github.com/ink1ing/prompter.git
cd prompter

# 编译项目
cargo build --release

# 启动历史监控（推荐）
./target/release/prompter --history-monitor
```

## 📊 监控界面预览

启动后显示专业监控面板：

```
============================================================
📚 CLAUDE HISTORY MONITOR v1.0
   Claude Code历史记录监控 - 直接捕获用户交互
============================================================
⏰ 启动时间: 2025-11-22 19:12:33
🔧 检查间隔: 2秒
------------------------------------------------------------
📊 监控状态总览
------------------------------------------------------------
📂 Claude历史文件:
   ✅ ~/.claude/history.jsonl (556599 bytes)

🎯 监控目标:
   📝 用户输入的中文提示词
   💬 Claude Code会话交互
   🏗️  项目上下文信息
------------------------------------------------------------
🚀 历史监控服务已就绪，监控Claude交互...
```

成功捕获提示词时的HTTP风格响应：

```
📥 [19:12:33.254] 200 OK - Claude历史交互已保存 | Records: 14
   └─ 📄 Content: 1.首先了解项目每个部分，这是一个定制化的自动推送的tgbo...
   └─ 🏗️  Project: /Users/inkling/Desktop/agent-daily
```

## 🛠️ 配置选项

### 基础使用

```bash
# 历史监控模式（推荐）
./target/release/prompter --history-monitor

# Shell监控模式
./target/release/prompter --shell-monitor

# 会话监控模式
./target/release/prompter --session-monitor

# 查看帮助
./target/release/prompter --help
```

### 自动上传模式

结合Cloudflare和GitHub自动上传：

```bash
# 历史监控 + 自动上传
./target/release/prompter --auto --history-monitor
```

配置 `config.toml`：

```toml
[app]
auto_upload_enabled = true
upload_interval_hours = 1

[filter]
detect_chinese = true
min_chinese_chars = 3

[cloudflare]
account_id = "your-account-id"
api_token = "your-api-token"

[website]
domain = "your-domain.com"
github_repo = "username/repo-name"
```

## 📁 数据文件

监控到的提示词保存位置：

- **历史监控**: `./data/claude_history_prompts.md`
- **Shell监控**: `./data/shell_captured_prompts.md`
- **会话监控**: `./data/claude_session_prompts.md`

文件格式示例：

```markdown
## 2025-11-22 19:12:33 (Claude历史监控)

**项目**: /Users/inkling/Desktop/agent-daily

1.首先了解项目每个部分，这是一个定制化的自动推送的tgbot
2.检查每日定时推送的逻辑是否严格生效，提高该逻辑的稳定性并测试
3.给用户一个tg内/push的指令，可以手动触发最近一次定时推送
```

## 🎨 启动脚本

| 脚本 | 功能 | 推荐场景 |
|------|------|----------|
| `click_me_stable.command` | 稳定版一键启动器 | **最推荐** - 双击即用 |
| `start_visual_monitor.sh` | 可视化Shell监控 | Shell监控专用 |
| `auto_start.sh` | 自动化配置向导 | 需要配置上传功能 |
| `demo_visual.sh` | 功能演示预览 | 了解功能特性 |

## 🔧 系统要求

- **操作系统**: macOS, Linux, Windows
- **Rust版本**: 1.70+
- **Shell**: bash, zsh, fish（用于Shell监控模式）
- **Claude Code**: 已安装并配置（用于历史监控模式）

## 📈 稳定版更新日志

### v2.0.0 (2025-11-22) - 稳定版

- ✅ **完全修复UTF-8字符边界问题** - 不再崩溃
- ✅ **新增历史监控模式** - 直接监控Claude Code内部历史
- ✅ **新增会话监控模式** - 进程级别监控
- ✅ **优化编译缓存处理** - 确保使用最新代码
- ✅ **新增稳定版启动器** - click_me_stable.command
- ✅ **系统状态检查功能** - 实时查看运行状态

### v1.0.0 (2024-11-21)

- ✨ 初始版本发布
- 🔍 独立Shell监控功能
- 🎨 可视化监控界面
- ☁️ Cloudflare自动上传
- 📱 GitHub集成推送

## ❓ 常见问题

**Q: 推荐使用哪种监控模式？**
A: 推荐使用**历史监控模式**（`--history-monitor`），最精准且100%稳定。

**Q: 为什么选择历史监控？**
A: 历史监控直接读取Claude Code官方历史文件（`~/.claude/history.jsonl`），捕获最准确，且已完全修复UTF-8问题。

**Q: 如何查看监控到的内容？**
A: 打开 `./data/claude_history_prompts.md` 文件即可查看所有捕获的中文提示词。

**Q: 程序会影响Claude Code使用吗？**
A: 完全不会！所有监控模式都是独立运行，不会影响Claude Code的使用体验。

**Q: UTF-8错误是否已解决？**
A: 是的，v2.0稳定版已完全解决所有UTF-8字符边界问题，可以安全处理任何长度的中文文本。

## 🤝 贡献

欢迎贡献代码、报告问题或提出建议！

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

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

🎯 **稳定版v2.0 - UTF-8完全修复 - 100%可靠运行**
