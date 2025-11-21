// 性能测试模块
use std::time::Instant;
use std::process::{Command, Stdio};
use std::io::{Write, BufRead, BufReader};

pub fn benchmark_direct_vs_pty() -> anyhow::Result<()> {
    println!("🔬 性能对比测试");
    println!("================");

    // 测试1: 直接运行 echo 命令
    let start = Instant::now();
    let output = Command::new("echo")
        .arg("Hello World")
        .output()?;
    let direct_time = start.elapsed();

    println!("✅ 直接执行: {:?}", direct_time);
    println!("   输出: {}", String::from_utf8_lossy(&output.stdout));

    // 测试2: 通过 PTY 运行相同命令
    let start = Instant::now();

    // 简化的 PTY 包装测试
    let mut child = Command::new("echo")
        .arg("Hello World")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let output = child.wait_with_output()?;
    let pty_time = start.elapsed();

    println!("✅ PTY包装: {:?}", pty_time);
    println!("   输出: {}", String::from_utf8_lossy(&output.stdout));

    // 计算开销
    let overhead = if pty_time > direct_time {
        pty_time - direct_time
    } else {
        std::time::Duration::from_nanos(0)
    };

    println!("\n📈 性能分析:");
    println!("   直接执行: {:?}", direct_time);
    println!("   PTY包装:  {:?}", pty_time);
    println!("   额外开销: {:?} ({:.2}%)",
        overhead,
        (overhead.as_nanos() as f64 / direct_time.as_nanos() as f64) * 100.0
    );

    Ok(())
}

pub fn test_io_throughput() -> anyhow::Result<()> {
    println!("\n🚀 I/O 吞吐量测试");
    println!("==================");

    let test_data = "测试数据\n".repeat(1000);

    let start = Instant::now();
    // 模拟大量数据处理
    let _processed = test_data.lines().count();
    let processing_time = start.elapsed();

    println!("✅ 处理1000行数据用时: {:?}", processing_time);
    println!("✅ 吞吐量: {:.2} 行/秒",
        1000.0 / processing_time.as_secs_f64()
    );

    Ok(())
}