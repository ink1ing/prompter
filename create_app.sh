#!/bin/bash

# 创建macOS应用包 (.app)

APP_NAME="Prompter"
APP_DIR="$APP_NAME.app"

echo "📱 创建macOS应用包..."

# 创建应用目录结构
mkdir -p "$APP_DIR/Contents/MacOS"
mkdir -p "$APP_DIR/Contents/Resources"

# 复制二进制文件
cp ./target/release/prompter "$APP_DIR/Contents/MacOS/"

# 创建Info.plist
cat > "$APP_DIR/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>prompter</string>
    <key>CFBundleIdentifier</key>
    <string>com.prompter.app</string>
    <key>CFBundleName</key>
    <string>Prompter</string>
    <key>CFBundleVersion</key>
    <string>1.0</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>LSUIElement</key>
    <true/>
</dict>
</plist>
EOF

# 创建启动脚本
cat > "$APP_DIR/Contents/MacOS/launcher.sh" << 'EOF'
#!/bin/bash
# 获取应用路径
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# 打开终端并运行prompter
osascript << 'APPLESCRIPT'
tell application "Terminal"
    activate
    do script "cd '$DIR' && ./prompter --simple; exit"
end tell
APPLESCRIPT
EOF

chmod +x "$APP_DIR/Contents/MacOS/launcher.sh"

echo "✅ macOS应用包已创建: $APP_DIR"
echo "💡 双击即可启动Prompter"