# Prompter 独立Shell监控使用指南

## 🎯 新功能概述

**独立Shell监控模式** 是Prompter的全新功能，允许你在**独立的终端**中启动监控服务，**无需PTY包装**，就能自动捕获其他终端中运行的Claude命令。

### ✨ 核心特性

- 🔍 **独立监控**: 在单独终端启动，监控所有其他终端
- 🚫 **无PTY包装**: 不影响Claude Code原生体验
- 🔄 **实时检测**: 每5秒扫描Shell历史文件变化
- 🌐 **多Shell支持**: 支持bash、zsh、fish等主流Shell
- 🇨🇳 **智能中文过滤**: 自动识别和保存中文提示词
- ☁️ **自动上传**: 可结合原有自动上传功能
- 📊 **GitHub集成**: 直接推送到GitHub仓库

## 🚀 快速开始

### 方法1: 使用自动化向导 (推荐)

```bash
# 运行Shell监控配置向导
./start_shell_monitor.sh
```

向导会：
- 自动编译项目
- 检查Shell历史文件
- 优化Shell配置
- 启动监控服务

### 方法2: 手动启动

```bash
# 编译项目
cargo build --release

# 启动独立Shell监控
./target/release/prompter --shell-monitor
```

## 🛠️ 工作原理

### 监控机制

Prompter通过监控Shell历史文件来捕获命令：

```
你在终端A运行: claude "帮我写一个函数"
      ↓
Shell历史文件更新
      ↓
Prompter监控器检测到变化
      ↓
提取并过滤中文内容
      ↓
保存到: ./data/shell_captured_prompts.md
```

### 支持的Shell历史格式

| Shell | 历史文件路径 | 格式特点 |
|-------|-------------|----------|
| **Bash** | `~/.bash_history` | 支持时间戳 |
| **Zsh** | `~/.zsh_history` | 扩展历史格式 |
| **Fish** | `~/.local/share/fish/fish_history` | YAML格式 |

## 📋 使用模式

### 1. 纯监控模式

```bash
# 只监控和保存，不上传
./target/release/prompter --shell-monitor
```

**特点**:
- 监控所有终端的Claude命令
- 保存中文提示词到本地文件
- 不自动上传，适合本地使用

### 2. 监控+自动上传模式

```bash
# 监控并自动上传到Cloudflare
./target/release/prompter --auto --shell-monitor
```

**特点**:
- 每小时自动上传中文提示词
- 同时支持PTY和Shell监控收集
- 推送到Cloudflare + GitHub

### 3. 后台服务模式

```bash
# 后台运行监控服务
nohup ./target/release/prompter --shell-monitor > shell_monitor.log 2>&1 &

# 查看后台进程
ps aux | grep prompter

# 停止后台服务
kill $(pgrep -f "prompter.*shell-monitor")
```

## ⚙️ Shell优化配置

为了获得最佳监控效果，建议优化Shell历史配置：

### Bash 配置 (~/.bashrc)

```bash
# 启用历史时间戳
export HISTTIMEFORMAT='%F %T '

# 增大历史记录数量
export HISTSIZE=10000
export HISTFILESIZE=20000

# 实时保存历史
shopt -s histappend
export PROMPT_COMMAND="history -a; history -c; history -r"
```

### Zsh 配置 (~/.zshrc)

```bash
# 启用扩展历史
setopt EXTENDED_HISTORY

# 增大历史记录数量
export HISTSIZE=10000
export SAVEHIST=10000

# 实时保存和共享历史
setopt INC_APPEND_HISTORY
setopt SHARE_HISTORY
```

### Fish 配置

Fish默认配置已经很好，无需额外优化。

## 📊 输出格式

### 本地文件格式

监控器会将捕获的中文提示词保存为Markdown格式：

```markdown
## 2024-11-21 14:30:45 (Shell监控)

```
claude "帮我写一个计算斐波那契数列的Python函数"
```

## 2024-11-21 14:35:22 (Shell监控)

```
claude "优化这段代码的性能，让它运行得更快"
```
```

### GitHub推送格式

如果启用自动上传，会推送JSON格式到GitHub：

```json
{
  "id": "batch-uuid-here",
  "timestamp": "2024-11-21T14:30:45+08:00",
  "prompts": [
    {
      "id": "prompt-uuid-here",
      "timestamp": "2024-11-21T14:30:45+08:00",
      "content": "claude \"帮我写一个函数\"",
      "chinese_chars": 12,
      "word_count": 20
    }
  ],
  "total_count": 1,
  "chinese_char_count": 12
}
```

## 🔧 配置文件

使用独立Shell监控时，可以通过`config.toml`进行详细配置：

```toml
[app]
auto_upload_enabled = true
upload_interval_hours = 1

[filter]
detect_chinese = true
min_chinese_chars = 3
exclude_commands = ["/", "quit", "exit", "help", "ls", "cd"]

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

## 📈 监控统计

### 实时状态信息

启动监控后，会显示：

```
🔍 启动独立Shell监控模式...
📂 监控的历史文件:
  ✅ /Users/username/.zsh_history
  ✅ /Users/username/.bash_history
🔍 扫描现有历史记录...
  📄 .zsh_history: 找到3条Claude命令
✅ 文件监控已启动，等待变化...
⏰ 启动定时扫描模式 (间隔: 5秒)
```

### 捕获通知

检测到新命令时：

```
🎯 捕获到中文Claude命令: 帮我写一个计算斐波那契数列的Python函数
📊 本轮检查发现1条新的Claude命令
```

## 🚨 故障排除

### 常见问题

**Q: 为什么没有检测到历史文件？**

A: 检查以下几点：
1. 确保使用的Shell启用了历史记录
2. 检查历史文件路径是否正确
3. 确保已经运行过Claude命令

**Q: 为什么捕获不到中文命令？**

A: 可能的原因：
1. 命令中中文字符少于3个（可在config.toml中调整）
2. 命令被排除规则过滤了
3. Shell历史还没有写入文件

**Q: 如何查看详细的监控日志？**

A: 运行带详细输出的监控：
```bash
./target/release/prompter --shell-monitor 2>&1 | tee monitor.log
```

### 手动测试

```bash
# 1. 检查历史文件是否存在
ls -la ~/.bash_history ~/.zsh_history ~/.local/share/fish/fish_history

# 2. 手动运行一个Claude命令
claude "测试中文提示词"

# 3. 检查历史文件末尾
tail -5 ~/.zsh_history

# 4. 查看Prompter输出目录
ls -la ./data/
```

## 🎯 最佳实践

### 1. 开发环境推荐配置

```bash
# 开发时使用监控模式
./target/release/prompter --shell-monitor

# 生产时使用自动上传
./target/release/prompter --auto --shell-monitor
```

### 2. 团队协作

```bash
# 团队成员都使用相同配置
cp config.toml.template config.toml
# 编辑个人的API密钥

# 统一推送到团队GitHub仓库
./target/release/prompter --auto --shell-monitor
```

### 3. 数据备份

```bash
# 定期备份收集的提示词
cp -r ./data/ ./backups/data-$(date +%Y%m%d)/
cp -r ./backups/ ~/Documents/prompter-backups/
```

## 🔄 与原有功能对比

| 特性 | PTY包装模式 | 独立Shell监控 |
|------|-------------|---------------|
| **启动方式** | 包装Claude命令 | 独立终端运行 |
| **对Claude影响** | 轻微延迟 | 零影响 |
| **实时性** | 实时捕获 | 5秒延迟 |
| **支持Shell** | 所有Shell | bash/zsh/fish |
| **历史命令** | 不支持 | 全支持 |
| **部署复杂度** | 简单 | 中等 |

## 🎉 完整使用流程

假设你想在macOS上使用zsh Shell监控Claude命令：

### 1. 初始化

```bash
# 克隆或下载Prompter项目
cd /path/to/prompter

# 运行自动化向导
./start_shell_monitor.sh
```

### 2. 配置确认

向导会提示输入：
- Cloudflare配置（如需自动上传）
- GitHub仓库信息
- 上传间隔设置

### 3. 启动监控

```bash
# 启动独立监控（纯本地）
./target/release/prompter --shell-monitor

# 或启动监控+自动上传
./target/release/prompter --auto --shell-monitor
```

### 4. 正常使用Claude

在任何其他终端中：

```bash
claude "帮我写一个快速排序算法"
claude "优化这段JavaScript代码的性能"
claude "设计一个用户登录系统的数据库结构"
```

### 5. 查看结果

```bash
# 查看本地保存的提示词
cat ./data/shell_captured_prompts.md

# 如果启用了自动上传，检查GitHub仓库
# 访问: https://github.com/your-username/your-repo
```

---

🎯 现在你就拥有了一个完全独立的Claude提示词监控系统！无需改变任何使用习惯，就能自动收集和管理所有的中文提示词。