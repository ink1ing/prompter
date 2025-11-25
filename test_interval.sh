#!/bin/bash
# Telegram反馈间隔功能快速测试脚本

echo "🧪 Telegram反馈间隔功能测试"
echo "================================"
echo ""

echo "📦 1. 编译项目..."
cargo build --release
if [ $? -ne 0 ]; then
    echo "❌ 编译失败"
    exit 1
fi
echo "✅ 编译成功"
echo ""

echo "🔧 2. 运行单元测试..."
cargo test --lib feedback_interval
if [ $? -ne 0 ]; then
    echo "⚠️ 测试失败，但继续..."
else
    echo "✅ 单元测试通过"
fi
echo ""

echo "📊 3. 测试间隔配置功能..."
echo ""

# 创建一个简单的Rust测试程序
cat > /tmp/test_interval.rs << 'EOF'
use anyhow::Result;
use prompter::feedback_interval::{FeedbackInterval, IntervalType};

fn main() -> Result<()> {
    println!("🧪 测试反馈间隔配置\n");

    // 测试1: 时间间隔
    println!("测试1: 设置时间间隔为12小时");
    let mut interval = FeedbackInterval::default();
    interval.set_time_based(12)?;
    println!("✅ {}\n", interval.get_config_description());

    // 测试2: 切换到数量间隔
    println!("测试2: 切换到数量间隔100个");
    interval.set_number_based(100)?;
    println!("✅ {}\n", interval.get_config_description());

    // 测试3: 增加计数
    println!("测试3: 模拟添加提示词");
    for i in 1..=5 {
        interval.increment_prompt_count()?;
        println!("  添加第{}个提示词", i);
    }
    println!("✅ {}\n", interval.get_config_description());

    // 测试4: 检查是否应该发送
    println!("测试4: 检查发送条件");
    let should_send = interval.should_send_feedback();
    println!("  当前计数: {}", interval.prompt_count_since_last);
    println!("  是否应该发送: {}\n", should_send);

    // 测试5: 再次切换回时间模式
    println!("测试5: 切换回时间间隔6小时");
    interval.set_time_based(6)?;
    println!("✅ {}\n", interval.get_config_description());

    println!("🎉 所有测试完成！");

    Ok(())
}
EOF

echo "📝 创建临时测试程序..."
rustc --edition 2021 \
    -L target/release/deps \
    --extern prompter=target/release/libprompter.rlib \
    --extern anyhow=target/release/deps/libanyhow-*.rlib \
    --extern chrono=target/release/deps/libchrono-*.rlib \
    --extern serde=target/release/deps/libserde-*.rlib \
    --extern serde_json=target/release/deps/libserde_json-*.rlib \
    --extern dirs=target/release/deps/libdirs-*.rlib \
    /tmp/test_interval.rs -o /tmp/test_interval 2>/dev/null

if [ -f /tmp/test_interval ]; then
    echo "✅ 测试程序编译成功"
    echo ""
    /tmp/test_interval
else
    echo "⚠️ 测试程序编译失败，使用cargo运行示例..."
    echo ""
    # 备用方案：使用示例程序
    cat > examples/quick_test.rs << 'EOFEX'
use anyhow::Result;
use prompter::feedback_interval::FeedbackInterval;

fn main() -> Result<()> {
    println!("🧪 快速测试反馈间隔配置\n");

    let mut interval = FeedbackInterval::default();

    println!("1️⃣ 时间模式: 12小时");
    interval.set_time_based(12)?;
    println!("{}\n", interval.get_config_description());

    println!("2️⃣ 数量模式: 100个提示词");
    interval.set_number_based(100)?;
    println!("{}\n", interval.get_config_description());

    println!("3️⃣ 增加5个提示词");
    for _ in 0..5 {
        interval.increment_prompt_count()?;
    }
    println!("{}\n", interval.get_config_description());

    println!("✅ 测试完成！");
    Ok(())
}
EOFEX

    cargo run --example quick_test 2>&1 | grep -v "warning:"
fi

echo ""
echo "================================"
echo "✅ 测试完成！"
echo ""
echo "💡 后续步骤："
echo "   1. 配置Telegram Bot: ./prompter --setup-feedback"
echo "   2. 启动监听器（需要实现）: ./prompter --telegram-listen"
echo "   3. 在Telegram中测试命令:"
echo "      /based-on-time 12"
echo "      /based-on-number 100"
echo "      /status"
echo ""
