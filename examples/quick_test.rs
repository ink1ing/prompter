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

    println!("4️⃣ 检查是否应该发送反馈");
    if interval.should_send_feedback() {
        println!("   ✅ 应该发送反馈");
    } else {
        println!("   ⏳ 还不需要发送反馈");
    }

    println!("\n✅ 测试完成！");
    Ok(())
}
