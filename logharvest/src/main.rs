//! logharvest —— 命令行入口
//!
//! 这个文件已经写好了，**你不需要修改它**（除非你想换输出格式，
//! 但那会让端到端测试失败）。它负责把三个实验的产物串起来：
//!
//! ```text
//! cargo run --release -- data/node-a.log data/node-b.log data/node-c.log
//! ```

use logharvest::collector::{benchmark, Summary};
use logharvest::{Level, Record};

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() || args.iter().any(|a| a == "-h" || a == "--help") {
        eprintln!("用法: logharvest <日志文件> [更多文件...]");
        eprintln!("例如: logharvest data/node-a.log data/node-b.log data/node-c.log");
        return if args.is_empty() {
            ExitCode::from(2)
        } else {
            ExitCode::SUCCESS
        };
    }

    let paths: Vec<PathBuf> = args.iter().map(PathBuf::from).collect();

    let (seq, seq_ms, con, con_ms) = benchmark(&paths);

    print_report(&con, &seq, seq_ms, con_ms);

    // 端到端正确性：两条路径的结果必须一致。
    // 这一条会直接反映在你的分数里。
    if !summaries_equal(&seq, &con) {
        eprintln!();
        eprintln!("错误：并发结果与单线程结果不一致，程序退出码为 1。");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

fn summaries_equal(a: &Summary, b: &Summary) -> bool {
    a.total_records == b.total_records
        && a.skipped_lines == b.skipped_lines
        && a.level_counts == b.level_counts
        && a.task_counts == b.task_counts
}

fn print_report(con: &Summary, seq: &Summary, seq_ms: u128, con_ms: u128) {
    let ratio = if con_ms == 0 {
        f64::INFINITY
    } else {
        seq_ms as f64 / con_ms as f64
    };

    println!("logharvest v0.1 — 并发日志采集分析器");
    println!("========================================");
    println!("采集文件: {} 个", con.per_file_lines.len());
    println!(
        "总记录数: {} (跳过脏行: {})",
        con.total_records, con.skipped_lines
    );
    println!(
        "采集耗时: {} ms (并发) / {} ms (单线程, {:.1}x 加速)",
        con_ms, seq_ms, ratio
    );
    println!();

    println!("级别分布:");
    let mut levels = [Level::Info, Level::Warn, Level::Error];
    levels.sort();
    for lv in levels {
        let n = con.level_counts.get(&lv).copied().unwrap_or(0);
        let pct = if con.total_records == 0 {
            0.0
        } else {
            n as f64 * 100.0 / con.total_records as f64
        };
        println!("  {:<5} {:>8}  ({:.1}%)", lv.label(), n, pct);
    }
    println!();

    println!("涉及任务: {} 个", con.task_counts.len());
    let top = con.top_tasks(3);
    if !top.is_empty() {
        let rendered: Vec<String> = top
            .iter()
            .map(|(id, c)| format!("#{} ({} 次)", id, c))
            .collect();
        println!("高频任务 ID: {}", rendered.join(", "));
    }

    // 单线程数字也打印一份，供你写简答题时抄录。
    println!();
    println!("[参考] 单线程共解析 {} 条，跳过 {} 条", seq.total_records, seq.skipped_lines);
}

/// 这个函数不会被 `main` 调用，它只是为了让 `Record` 的 `Display`
/// 未来有地方可用，同时避免死代码警告。
#[allow(dead_code)]
fn format_record(r: &Record) -> String {
    format!(
        "{} {} [{}] {}",
        r.timestamp,
        r.level.label(),
        r.source,
        r.message
    )
}
