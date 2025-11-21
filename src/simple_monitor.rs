// 简化版本：直接监控stdin输入
use std::io::{self, Write};
use std::fs::OpenOptions;
use chrono::Local;

pub fn run_simple_monitor() -> io::Result<()> {
    println!("🚀 Simple Prompter - 输入提示词将自动保存");
    println!("📝 保存文件: prompts.md");
    println!("💡 输入 'quit' 退出\n");

    let stdin = io::stdin();

    loop {
        print!("Your prompt: ");
        io::stdout().flush()?;

        let mut input = String::new();
        stdin.read_line(&mut input)?;

        let trimmed = input.trim();

        if trimmed == "quit" {
            break;
        }

        if !trimmed.is_empty() {
            save_prompt(trimmed)?;
            println!("✅ 已保存: {}", trimmed);
        }
    }

    println!("👋 Goodbye!");
    Ok(())
}

fn save_prompt(prompt: &str) -> io::Result<()> {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let content = format!("## {}\n\n```\n{}\n```\n\n", timestamp, prompt);

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("prompts.md")?;

    file.write_all(content.as_bytes())?;
    Ok(())
}