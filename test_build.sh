#!/bin/bash

echo "🔨 编译 Prompter..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ 编译成功！"
    echo ""
    echo "🚀 测试简化版本:"
    echo "./target/release/prompter --simple"
    echo ""
    echo "🚀 测试完整版本（需要Claude Code CLI）:"
    echo "./target/release/prompter"
    echo ""
    echo "📖 查看帮助:"
    echo "./target/release/prompter --help"
else
    echo "❌ 编译失败"
    exit 1
fi