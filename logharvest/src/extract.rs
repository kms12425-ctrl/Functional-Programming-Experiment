//! 实验二：字段提取与统计
//!
//! 任务：实现 [`extract_task_id`] 和 [`count_by_level`]。
//!
//! [`find_field`] 已经给你了，你要基于它往上搭。
//!
//! 完整说明见《Rust 上机实验报告》实验二部分。

use crate::parser::{Level, Record};
use std::collections::HashMap;

/// 从一段文本里找出 `key=value` 形式的字段，返回 value 的**借用**切片。
///
/// 这个函数已经给你了，**不需要你实现**。
/// 但它出现在这里是有意的：请仔细看它的签名，特别是那个 `<'a>`。
///
/// ```text
/// fn find_field<'a>(message: &'a str, key: &str) -> Option<&'a str>
///                          ^^              ^^^^             ^^
///                          |                |               |
///                       输入的生命周期   普通输入       返回值的生命周期
/// ```
///
/// 你会疑惑：为什么 `key` 没有标生命周期，而 `message` 和返回值标了同一个 `'a`？
/// 这个问题的答案在《Rust 上机实验报告》实验二的作答区里，请把它写清楚。
///
/// 行为说明：
/// - 只匹配「键名完整相等」的字段，`taskid=1` 不会匹配 `id`。
/// - 找不到时返回 `None`，不 panic。
/// - 返回的是 `message` 内部的一段切片，不复制。
///
/// # 例子
///
/// ```
/// use logharvest::find_field;
/// assert_eq!(find_field("task started id=1024", "id"), Some("1024"));
/// assert_eq!(find_field("task started id=1024", "taskid"), None);
/// assert_eq!(find_field("task failed reason=timeout", "id"), None);
/// ```
pub fn find_field<'a>(message: &'a str, key: &str) -> Option<&'a str> {
    // 按空白切分，对每一段尝试剥离 `key=` 前缀。
    for token in message.split_whitespace() {
        if let Some(value) = token.strip_prefix(key) {
            if let Some(value) = value.strip_prefix('=') {
                return Some(value);
            }
        }
    }
    None
}

/// 从一条记录的消息正文里抽出任务 ID。
///
/// # 你要实现的部分
///
/// TODO(实验二 a): 实现这个函数，大约 8 行。
///
/// 要求：
/// - 借助 [`find_field`] 找到 `id` 字段；
/// - 把找到的字符串解析成 `u64`；
/// - `id` 不存在、或者存在但不是合法数字，**都返回 `None`**。
///
/// 这里有两种**不同来源**的失败，你要把它们归约成同一个 `None`：
/// 1. `find_field` 返回 `Option<&str>` —— 字段压根不存在；
/// 2. `str::parse::<u64>()` 返回 `Result<u64, _>` —— 字段存在但不是数字。
///
/// 提示方向：`?` 对 `Option` 和 `Result` 都能用，但在同一个函数里
/// 不能直接混用。想一想怎么在其中一步做个转换才最自然。
///
/// 数据集里有一部分行**格式完全合法、但 id 的值不是数字**
/// （例如 `task lookup id=unknown`），它们就是用来验证你处理了第 2 种失败的。
///
/// # 例子
///
/// ```
/// use logharvest::{extract_task_id, Level, Record};
/// let r = Record::new("2026-09-19T10:23:45Z", Level::Info, "worker-3", "task started id=1024");
/// assert_eq!(extract_task_id(&r), Some(1024));
///
/// let r = Record::new("2026-09-19T10:23:45Z", Level::Info, "worker-3", "task started");
/// assert_eq!(extract_task_id(&r), None);
/// ```
pub fn extract_task_id(record: &Record) -> Option<u64> {
    todo!("实验二 a：实现 extract_task_id")
}

/// 按日志级别统计记录条数。
///
/// # 你要实现的部分
///
/// TODO(实验二 b): 实现这个函数，大约 10 行。
///
/// 要求：
/// - 返回一个 `HashMap<Level, usize>`，键是出现过的级别；
/// - **没有出现过的级别不要放进 map**，也就是说空输入返回空 map，
///   而不是一个三个键都是 0 的 map；
/// - 用 `entry` API 累加。
///
/// 注意：[`Level`] 的 `Hash` 和 `Eq` 已经 derive 好了，直接当键用即可。
///
/// # 例子
///
/// ```
/// use logharvest::{count_by_level, Level, Record};
/// let rs = vec![
///     Record::new("t", Level::Info, "s", "m"),
///     Record::new("t", Level::Info, "s", "m"),
///     Record::new("t", Level::Error, "s", "m"),
/// ];
/// let c = count_by_level(&rs);
/// assert_eq!(c.get(&Level::Info), Some(&2));
/// assert_eq!(c.get(&Level::Error), Some(&1));
/// assert_eq!(c.get(&Level::Warn), None);
/// ```
pub fn count_by_level(records: &[Record]) -> HashMap<Level, usize> {
    todo!("实验二 b：实现 count_by_level")
}
