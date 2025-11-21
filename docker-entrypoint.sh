#!/bin/bash

# Docker 容器启动脚本

echo "🐳 Prompter Docker Container"
echo "============================"

# 设置输出目录
export OUTPUT_DIR=${OUTPUT_DIR:-/data}

# 启动prompter
exec prompter -o "$OUTPUT_DIR/prompts.md" "$@"