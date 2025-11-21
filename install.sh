#!/bin/bash

# Prompter 安装脚本 - 提供多种启动方式

echo "🚀 Prompter 安装向导"
echo "===================="

# 检查编译状态
if [ ! -f "./target/release/prompter" ]; then
    echo "📦 编译项目..."
    cargo build --release
fi

echo ""
echo "请选择安装方式："
echo "1) 全局命令安装（推荐）"
echo "2) 桌面快捷方式"
echo "3) 系统服务安装"
echo "4) 创建启动脚本"

read -p "请输入选择 (1-4): " choice

case $choice in
    1)
        # 全局命令安装
        echo "🔧 安装全局命令..."
        sudo cp ./target/release/prompter /usr/local/bin/
        sudo chmod +x /usr/local/bin/prompter
        echo "✅ 安装完成！现在可以在任何地方运行 'prompter' 命令"
        echo "💡 试试: prompter --help"
        ;;
    2)
        # 桌面快捷方式
        echo "🖥️  创建桌面快捷方式..."
        DESKTOP_FILE="$HOME/Desktop/Prompter.app"
        mkdir -p "$DESKTOP_FILE/Contents/MacOS"

        cat > "$DESKTOP_FILE/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>prompter</string>
    <key>CFBundleName</key>
    <string>Prompter</string>
    <key>CFBundleVersion</key>
    <string>1.0</string>
</dict>
</plist>
EOF

        cp ./target/release/prompter "$DESKTOP_FILE/Contents/MacOS/"
        echo "✅ 桌面快捷方式已创建"
        ;;
    3)
        # 系统服务安装
        echo "⚙️  创建系统服务..."
        cat > prompter.service << EOF
[Unit]
Description=Prompter - Claude Code Monitor
After=network.target

[Service]
Type=simple
User=$USER
WorkingDirectory=$(pwd)
ExecStart=$(pwd)/target/release/prompter --simple
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF
        echo "✅ 服务文件已创建: prompter.service"
        echo "💡 使用方法:"
        echo "   sudo cp prompter.service /etc/systemd/system/"
        echo "   sudo systemctl enable prompter"
        echo "   sudo systemctl start prompter"
        ;;
    4)
        # 创建启动脚本
        echo "📝 创建启动脚本..."
        cat > start_prompter.sh << 'EOF'
#!/bin/bash

# Prompter 启动脚本
echo "🎯 Prompter 启动选项"
echo "=================="
echo ""
echo "1) 简化模式（直接输入）"
echo "2) PTY监控模式"
echo "3) 性能测试模式"
echo "4) 自定义参数"
echo ""

read -p "请选择模式 (1-4): " mode

case $mode in
    1)
        echo "🚀 启动简化模式..."
        ./target/release/prompter --simple
        ;;
    2)
        echo "🔍 启动PTY监控模式..."
        ./target/release/prompter
        ;;
    3)
        echo "📊 运行性能测试..."
        ./target/release/prompter --benchmark
        ;;
    4)
        read -p "输入自定义参数: " params
        echo "🛠️  启动自定义模式..."
        ./target/release/prompter $params
        ;;
esac
EOF
        chmod +x start_prompter.sh
        echo "✅ 启动脚本已创建: start_prompter.sh"
        ;;
esac

echo ""
echo "🎉 安装完成！"