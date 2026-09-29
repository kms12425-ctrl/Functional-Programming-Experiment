//! 实验二：`extract_task_id` 与 `count_by_level` 的正确性测试。
//!
//! 跑法：`cargo test --test lab02_extract`

use logharvest::{count_by_level, extract_task_id, find_field, Level, Record};

fn rec(msg: &str) -> Record {
    Record::new("2026-09-19T10:23:45Z", Level::Info, "worker-3", msg)
}

// --- find_field：脚手架已给，这里只是锁定它的行为，学生不用改 ---

#[test]
fn find_field_basic() {
    assert_eq!(find_field("task started id=1024", "id"), Some("1024"));
    assert_eq!(find_field("task failed reason=timeout", "reason"), Some("timeout"));
}

#[test]
fn find_field_requires_exact_key() {
    assert_eq!(find_field("taskid=1 id=2", "id"), Some("2"));
    assert_eq!(find_field("taskid=1", "id"), None);
}

#[test]
fn find_field_absent() {
    assert_eq!(find_field("task started", "id"), None);
    assert_eq!(find_field("", "id"), None);
}

// --- extract_task_id ---

#[test]
fn extract_id_normal() {
    assert_eq!(extract_task_id(&rec("task started id=1024")), Some(1024));
    assert_eq!(extract_task_id(&rec("任务开始处理 id=256")), Some(256));
}

#[test]
fn extract_id_multiple_takes_first() {
    assert_eq!(extract_task_id(&rec("id=512 then id=1024")), Some(512));
}

#[test]
fn extract_id_absent_is_none() {
    assert_eq!(extract_task_id(&rec("task started")), None);
    assert_eq!(extract_task_id(&rec("reason=timeout")), None);
}

#[test]
fn extract_id_non_numeric_is_none() {
    assert_eq!(extract_task_id(&rec("task lookup id=unknown")), None);
    assert_eq!(extract_task_id(&rec("id=abc123")), None);
}

#[test]
fn extract_id_does_not_panic_on_garbage() {
    for m in ["id=", "id==1", "id", "=1", "id=-5", "id=999999999999999999999999"] {
        let _ = extract_task_id(&rec(m));
    }
}

// --- count_by_level ---

#[test]
fn count_empty_returns_empty_map() {
    assert!(count_by_level(&[]).is_empty());
}

#[test]
fn count_mixed_levels() {
    let rs = vec![
        Record::new("t", Level::Info, "s", "m"),
        Record::new("t", Level::Info, "s", "m"),
        Record::new("t", Level::Warn, "s", "m"),
        Record::new("t", Level::Error, "s", "m"),
        Record::new("t", Level::Error, "s", "m"),
        Record::new("t", Level::Error, "s", "m"),
    ];
    let c = count_by_level(&rs);
    assert_eq!(c.get(&Level::Info), Some(&2));
    assert_eq!(c.get(&Level::Warn), Some(&1));
    assert_eq!(c.get(&Level::Error), Some(&3));
}

#[test]
fn count_omits_absent_levels() {
    let rs = vec![Record::new("t", Level::Info, "s", "m")];
    let c = count_by_level(&rs);
    assert_eq!(c.get(&Level::Warn), None);
    assert_eq!(c.get(&Level::Error), None);
    assert_eq!(c.len(), 1);
}
