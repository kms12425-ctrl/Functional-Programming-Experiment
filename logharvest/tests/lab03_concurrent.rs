//! 实验三：`collect_concurrent` 的并发正确性测试，以及端到端测试。
//!
//! 跑法：`cargo test --test lab03_concurrent`
//!
//! 这些测试的关键断言是：**并发结果必须和单线程结果逐字段相等**。
//! 只靠"能跑出数字"是拿不到的。

use logharvest::{collect_concurrent, collect_sequential, Level, Summary};
use std::path::PathBuf;

fn data(names: &[&str]) -> Vec<PathBuf> {
    names.iter().map(|n| PathBuf::from("data").join(n)).collect()
}

fn assert_same(a: &Summary, b: &Summary, ctx: &str) {
    assert_eq!(a.total_records, b.total_records, "{}: total_records", ctx);
    assert_eq!(a.skipped_lines, b.skipped_lines, "{}: skipped_lines", ctx);
    assert_eq!(a.level_counts, b.level_counts, "{}: level_counts", ctx);
    assert_eq!(a.task_counts, b.task_counts, "{}: task_counts", ctx);
    assert_eq!(a.per_file_lines, b.per_file_lines, "{}: per_file_lines", ctx);
}

#[test]
fn concurrent_matches_sequential_single_file() {
    let p = data(&["node-a.log"]);
    assert_same(&collect_sequential(&p), &collect_concurrent(&p), "1 file");
}

#[test]
fn concurrent_matches_sequential_three_files() {
    let p = data(&["node-a.log", "node-b.log", "node-c.log"]);
    assert_same(&collect_sequential(&p), &collect_concurrent(&p), "3 files");
}

#[test]
fn concurrent_matches_sequential_many_files() {
    // 重复喂同样的文件，压一下线程数量。
    let p = data(&[
        "node-a.log", "node-b.log", "node-c.log", "node-a.log", "node-b.log", "node-c.log",
        "node-a.log", "node-b.log", "node-c.log", "node-c.log",
    ]);
    assert_same(&collect_sequential(&p), &collect_concurrent(&p), "10 files");
}

#[test]
fn per_file_lines_follows_input_order() {
    // 顺序是语义的一部分。
    // 注意：`per_file_lines` 统计的是**非空行**，空行在解析前就被跳过，
    // 不算 dirty。三个文件的非空行数分别是 897 / 1099 / 695。
    let p = data(&["node-a.log", "node-c.log"]);
    let s = collect_concurrent(&p);
    assert_eq!(s.per_file_lines[0], 897, "第一个应该是 node-a（897 非空行）");
    assert_eq!(s.per_file_lines[1], 695, "第二个应该是 node-c（695 非空行）");
}

#[test]
fn per_file_lines_order_matters_and_is_preserved() {
    // 正序和逆序的结果里，per_file_lines 应该正好相反 —— 证明顺序真的被保留了。
    let fwd = collect_concurrent(&data(&["node-a.log", "node-c.log"]));
    let rev = collect_concurrent(&data(&["node-c.log", "node-a.log"]));
    assert_eq!(fwd.per_file_lines, vec![897, 695]);
    assert_eq!(rev.per_file_lines, vec![695, 897]);
    // 但总数与级别分布必须一致
    assert_eq!(fwd.total_records, rev.total_records);
    assert_eq!(fwd.level_counts, rev.level_counts);
}

#[test]
fn empty_input_is_ok() {
    let s = collect_concurrent(&[]);
    assert_eq!(s.total_records, 0);
    assert_eq!(s.skipped_lines, 0);
    assert!(s.per_file_lines.is_empty());
}

#[test]
fn missing_file_does_not_panic() {
    let p = vec![PathBuf::from("data/this-does-not-exist.log")];
    let s = collect_concurrent(&p);
    assert_eq!(s.total_records, 0);
    // 与单线程保持一致
    assert_same(&collect_sequential(&p), &s, "missing file");
}

#[test]
fn dirty_lines_are_counted_not_panicked() {
    let p = data(&["node-a.log", "node-b.log", "node-c.log"]);
    let s = collect_concurrent(&p);
    assert!(s.skipped_lines > 0, "数据集里埋了脏行，skipped 必须大于 0");
    assert!(s.total_records > 0);
}

#[test]
fn end_to_end_known_numbers() {
    // 数据集是固定种子生成的，这些数字是稳定的。
    // 这一条同时验证 read_file + parse + extract + merge 全链路。
    //
    // 数据集的精确构成（可用 scripts/gen_data.py 复现）：
    //   node-a.log 900 行（含 3 空行 → 897 非空）
    //   node-b.log 1100 行（含 1 空行 → 1099 非空）
    //   node-c.log 700 行（含 5 空行 → 695 非空）
    //   合计 2700 行，其中 9 行是空白（被跳过、不计入 dirty）
    let p = data(&["node-a.log", "node-b.log", "node-c.log"]);
    let s = collect_concurrent(&p);
    assert_eq!(
        s.per_file_lines.iter().sum::<usize>(),
        2691,
        "非空行总数应为 2691"
    );
    assert_eq!(s.per_file_lines, vec![897, 1099, 695], "顺序与逐文件行数");
    // 脏行 + 正常行 = 非空行总数
    assert_eq!(
        s.total_records + s.skipped_lines,
        2691,
        "每条非空行要么被解析、要么被判脏，不能凭空消失"
    );
    // 解析失败比例应该在合理区间：脏行率是 2%~5%，加上 id 非法但格式合法的行
    let dirty_ratio = s.skipped_lines as f64 / 2691.0;
    assert!(
        (0.01..0.10).contains(&dirty_ratio),
        "脏行率 {} 超出预期区间，检查解析规则或数据生成",
        dirty_ratio
    );
    assert!(s.level_counts.get(&Level::Info).copied().unwrap_or(0) > 0, "应该有 INFO");
    assert!(s.level_counts.get(&Level::Warn).copied().unwrap_or(0) > 0, "应该有 WARN");
    assert!(s.level_counts.get(&Level::Error).copied().unwrap_or(0) > 0, "应该有 ERROR");
    assert!(!s.task_counts.is_empty(), "应该抽出过任务 ID");
}

#[test]
fn top_tasks_is_sorted_and_stable() {
    let p = data(&["node-a.log", "node-b.log", "node-c.log"]);
    let s = collect_concurrent(&p);
    let top = s.top_tasks(3);
    assert!(top.len() <= 3);
    // 次数必须单调不增
    for w in top.windows(2) {
        assert!(w[0].1 >= w[1].1, "top_tasks 必须按次数降序");
    }
    // 稳定性：跑两次结果一样
    let s2 = collect_concurrent(&p);
    assert_eq!(s.top_tasks(3), s2.top_tasks(3));
}
