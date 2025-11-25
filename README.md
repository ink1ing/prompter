# 🤖 Prompter

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)
[![Stable: v3.0](https://img.shields.io/badge/stable-v3.0-green.svg?style=for-the-badge)](https://github.com/ink1ing/prompter)

**智能Claude提示词分析系统 - v3.0 AI增强版**

Prompter 是一个用 Rust 编写的开源AI反馈系统，不仅能监控和收集 Claude Code CLI 的中文提示词，更能通过Google Gemini思考模型提供专业的优化建议，让每个人都能轻松提升AI交互质量。

## ✨ v3.0 AI增强版特性

### 🧠 AI智能分析
- ✅ **Google Gemini思考模型** - 深度分析提示词质量和结构
- ✅ **自动降级机制** - 配额不足时智能切换快速模型
- ✅ **个性化建议** - 基于您的使用习惯提供定制优化方案
- ✅ **Telegram推送** - 每日AI分析报告直接发送到手机

### 🚀 一键使用体验
- ✅ **双击启动** - 运行 `start.sh` 选择功能即可
- ✅ **自动配置** - 浏览器自动打开引导获取API密钥
- ✅ **零门槛使用** - 无需技术背景，普通用户也能轻松使用
- ✅ **完全开源** - 代码透明，API密钥仅存储在本地

## 🚀 快速开始

### 方式一：一键启动（推荐）

1. 下载项目文件
2. 双击运行 `start.sh`
3. 选择 `1. 一键配置系统`
4. 按提示完成API配置
5. 开始享受AI优化建议！

### 方式二：从源码编译

```bash
# 克隆项目
git clone https://github.com/ink1ing/prompter.git
cd prompter

# 编译项目
cargo build --release

# 运行一键启动器
./start.sh
```

## 📋 功能菜单

运行 `./start.sh` 后可选择：

1. 🚀 **一键配置系统** - 首次使用推荐（自动打开浏览器引导）
2. 📊 **启动历史监控** - 监控Claude Code历史记录
3. 🔍 **启动会话监控** - 实时监控Claude进程交互
4. 🤖 **启动AI分析服务** - 自动分析并推送优化建议
5. 🧪 **测试系统配置** - 检查API连接状态
6. 📖 **查看帮助文档** - 完整使用指南
7. 🎨 **演示系统功能** - 查看功能演示

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

## 🧠 AI智能分析功能

### 一键配置

```bash
./target/release/prompter --setup-feedback
```

系统将自动：
1. 🌐 **打开Google AI Studio** - 帮助您获取Gemini API密钥
2. 📱 **打开Telegram BotFather** - 指导创建专属机器人
3. 🔑 **自动获取Chat ID** - 无需手动查找
4. 🧪 **测试API连接** - 确保配置正确

### 配置文件

首次配置后，系统自动生成 `config.toml`：

```toml
[ai_feedback]
enabled = true                    # 系统启用状态
gemini_api_key = "YOUR_KEY"       # 您的Gemini API密钥
telegram_bot_token = "YOUR_TOKEN" # 您的Telegram Bot Token
telegram_chat_id = "YOUR_ID"      # 自动获取的Chat ID
daily_report_time = "09:00"       # 每日推送时间
max_prompts_per_analysis = 50     # 单次分析最大提示词数量

# Gemini模型配置
thinking_model = "models/gemini-2.0-flash-thinking-exp"    # 思考模型
fast_model = "models/gemini-2.5-flash"                     # 快速模型
thinking_budget = 1024                                     # 思考预算
auto_fallback = true                                       # 自动降级
max_retries = 3                                            # 最大重试次数
```

### AI分析工作流程

```
1. 📊 收集提示词 → 监控Claude交互
2. 🧠 思考模型分析 → 深度理解提示词意图
3. 🔍 检测配额 → 429错误自动检测
4. 🔄 智能重试 → 指数退避算法
5. ⚡ 快速模型降级 → 确保服务可用
6. 📱 Telegram推送 → 每日优化建议
```

### 思考预算配置

- **`-1` (动态思考)**: AI自动调整思考深度，推荐使用
- **`0` (关闭思考)**: 仅使用快速模型，节省配额
- **`>0` (固定预算)**: 指定Token数量，如 `1024`

## 🛠️ 监控模式

### 历史监控模式 ⭐ 推荐

```bash
./target/release/prompter --history-monitor
```

直接读取 `~/.claude/history.jsonl`，最精准的监控方式。

### 会话监控模式

```bash
./target/release/prompter --session-monitor
```

实时监控活跃的Claude Code进程交互。

### Shell监控模式

```bash
./target/release/prompter --shell-monitor
```

监控Shell历史文件中的Claude命令。

### AI分析服务

```bash
./target/release/prompter --auto-feedback
```

启动每日AI分析服务，自动推送优化建议到Telegram。

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
| `start.sh` | v3.0一键启动器 | **最推荐** - 包含所有功能 |
| `demo_ai_feedback.sh` | AI功能演示 | 了解AI分析特性 |
| `click_me_stable.command` | v2.0稳定版启动器 | 仅需监控功能 |

## 🔧 系统要求

- **操作系统**: macOS, Linux, Windows
- **Rust版本**: 1.70+
- **Shell**: bash, zsh, fish（用于Shell监控模式）
- **Claude Code**: 已安装并配置（用于历史监控模式）

## 📈 版本更新日志

### v3.0.0 (2025-11-23) - AI增强版 🆕

- 🧠 **Google Gemini AI分析** - 思考模型深度分析提示词
- ⚡ **智能降级机制** - 配额不足自动切换快速模型
- 📱 **Telegram推送** - 每日AI优化建议
- 🚀 **一键配置向导** - 自动打开浏览器引导
- 🔐 **隐私保护** - API密钥仅存储在本地
- 🎯 **零门槛使用** - 普通用户也能轻松上手
- 📊 **思考预算控制** - 动态/固定/关闭三种模式

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

### 🧠 AI功能相关

**Q: 需要付费吗？**
A: Gemini API和Telegram Bot都提供免费额度，日常使用完全免费。配额不足时系统会自动降级到快速模型。

**Q: API密钥安全吗？**
A: 完全安全！所有API密钥仅存储在您本地的 `config.toml` 文件中，不会上传到任何服务器。

**Q: 如何获取API密钥？**
A: 运行 `./start.sh` 选择"一键配置系统"，程序会自动打开浏览器引导您获取。

**Q: Gemini配额用完了怎么办？**
A: 系统会自动检测429错误并切换到快速模型，确保服务持续可用。

**Q: 如何修改每日推送时间？**
A: 编辑 `config.toml` 中的 `daily_report_time` 参数，如 `"09:00"` 改为 `"18:00"`。

### 📊 监控功能相关

**Q: 推荐使用哪种监控模式？**
A: 推荐使用**历史监控模式**（`--history-monitor`），最精准且100%稳定。

**Q: 为什么选择历史监控？**
A: 历史监控直接读取Claude Code官方历史文件（`~/.claude/history.jsonl`），捕获最准确，且已完全修复UTF-8问题。

**Q: 如何查看监控到的内容？**
A: 打开 `./data/claude_history_prompts.md` 文件即可查看所有捕获的提示词。

**Q: 程序会影响Claude Code使用吗？**
A: 完全不会！所有监控模式都是独立运行，不会影响Claude Code的使用体验。

### 🔧 技术问题

**Q: UTF-8错误是否已解决？**
A: 是的，v2.0+版本已完全解决所有UTF-8字符边界问题，可以安全处理任何长度的中文文本。

**Q: 浏览器没有自动打开？**
A: 手动访问以下链接：
- Gemini API: https://aistudio.google.com/api-keys
- Telegram Bot: https://t.me/BotFather

**Q: 无法获取Chat ID？**
A: 请确保您已向Telegram机器人发送过至少一条消息（如 `/start`），然后重新运行配置。

**Q: 如何查看日志？**
A: 运行 `tail -f prompter.log` 查看实时日志，或运行 `./prompter --test-feedback` 进行诊断。

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

<div align="center">

## 🌟 如果这个项目对您有帮助，请给我们一个 Star！

**v3.0 AI增强版 - 让每个人都能优化自己的AI提示词**

📚 [文档](https://github.com/ink1ing/prompter/wiki) •
🐛 [问题反馈](https://github.com/ink1ing/prompter/issues) •
💬 [讨论](https://github.com/ink1ing/prompter/discussions)

Built with ❤️ using Rust + Google Gemini + Telegram

</div>
