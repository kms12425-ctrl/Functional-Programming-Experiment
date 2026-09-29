//! 实验一：`parse_line` 的正确性测试。
//!
//! 跑法：`cargo test --test lab01_parser`

use logharvest::{parse_line, Level, Record};

#[test]
fn parses_a_well_formed_line() {
    let r = parse_line("2026-09-19T10:23:45Z INFO  [worker-3] task started id=1024")
        .expect("合法行必须能被解析");
    assert_eq!(
        r,
        Record::new(
            "2026-09-19T10:23:45Z",
            Level::Info,
            "worker-3",
            "task started id=1024"
        )
    );
}

#[test]
fn parses_all_three_levels() {
    for (s, want) in [("INFO", Level::Info), ("WARN", Level::Warn), ("ERROR", Level::Error)] {
        let line = format!("2026-09-19T10:23:45Z {} [worker-1] hello", s);
        let r = parse_line(&line).unwrap_or_else(|| panic!("{} 应该能解析", s));
        assert_eq!(r.level, want);
    }
}

#[test]
fn keeps_spaces_inside_message() {
    let r = parse_line("2026-09-19T10:23:47Z ERROR [worker-3] task failed id=1024 reason=timeout")
        .unwrap();
    assert_eq!(r.message, "task failed id=1024 reason=timeout");
    assert_eq!(r.source, "worker-3");
}

#[test]
fn rejects_malformed_line() {
    assert!(parse_line("malformed line without structure").is_none());
}

#[test]
fn rejects_unknown_level() {
    assert!(parse_line("2026-09-19T10:23:45Z TRACE [worker-3] nope").is_none());
}

#[test]
fn rejects_missing_brackets() {
    assert!(parse_line("2026-09-19T10:23:45Z INFO worker-3 no brackets").is_none());
}

#[test]
fn rejects_bad_timestamp() {
    // 形状像时间戳但不合法，必须被拒。这一条最容易漏。
    assert!(parse_line("2026-13-45T99:99:99Z INFO [worker-3] bad ts").is_none());
}

#[test]
fn rejects_non_timestamp_first_field() {
    assert!(parse_line("not a timestamp INFO [worker-3] hi").is_none());
}

#[test]
fn rejects_too_few_fields() {
    assert!(parse_line("2026-09-19T10:23:45Z INFO").is_none());
    assert!(parse_line("").is_none());
}

#[test]
fn does_not_panic_on_garbage() {
    // 核心约束：任何输入都不许 panic。这里喂一堆奇怪的东西。
    for s in [
        " ",
        "\t",
        "[[[",
        "=====",
        "2026-09-19T10:23:45Z INFO [ ] x",
        "2026-09-19T10:23:45Z INFO [unclosed x",
        "中文 日志 没有 结构",
        &"a".repeat(10_000),
    ] {
        let _ = parse_line(s); // 只要不 panic 就行
    }
}

#[test]
fn handles_chinese_message() {
    let r = parse_line("2026-09-19T10:23:45Z WARN  [reaper] 任务失败 id=512 reason=timeout").unwrap();
    assert_eq!(r.message, "任务失败 id=512 reason=timeout");
    assert_eq!(r.source, "reaper");
}
