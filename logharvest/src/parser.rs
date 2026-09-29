//! 实验一：日志行解析器
//!
//! 任务：实现 [`parse_line`]，把一行原始日志文本变成结构化的 [`Record`]。
//!
//! 核心约束：**解析失败必须返回 `None`，不许 panic。**
//! 真实日志里必然有格式不合规的行，一个遇到脏行就崩溃的解析器在工程上没有价值。
//!
//! 完整判定规则见《Rust 上机实验报告》实验一部分。

use std::fmt;

/// 日志级别。
///
/// `Hash` 和 `Eq` 已经 derive 好了，实验二里可以直接把 `Level` 当作
/// `HashMap` 的键来用，不需要你额外操心。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Level {
    Info,
    Warn,
    Error,
}

impl Level {
    /// 从字符串映射到级别。只接受三个大写形式，其余返回 `None`。
    ///
    /// 这个函数已经给你了，实验一里你不需要改它。
    pub fn from_str_strict(s: &str) -> Option<Level> {
        match s {
            "INFO" => Some(Level::Info),
            "WARN" => Some(Level::Warn),
            "ERROR" => Some(Level::Error),
            _ => None,
        }
    }

    /// 用于展示的大写形式，宽度补齐到 5，方便输出对齐。
    pub fn label(self) -> &'static str {
        match self {
            Level::Info => "INFO ",
            Level::Warn => "WARN ",
            Level::Error => "ERROR",
        }
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label().trim_end())
    }
}

/// 一条结构化的日志记录。
///
/// 字段全部是 `String`（拥有所有权），不是 `&str`（借用）。
/// 这一点是有意设计的：记录要跨越函数边界、最终跨越线程边界，
/// 它必须自己拥有数据，不能依赖别人的生命周期。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    /// RFC3339 形式的时间戳，例如 `2026-09-19T10:23:45Z`
    pub timestamp: String,
    /// 日志级别
    pub level: Level,
    /// 来源标识，例如 `worker-3`（注意：方括号已经被剥掉）
    pub source: String,
    /// 剩余的消息正文，例如 `task started id=1024`
    pub message: String,
}

impl Record {
    /// 构造一条记录。实验二、实验三的测试会用到。
    pub fn new(
        timestamp: impl Into<String>,
        level: Level,
        source: impl Into<String>,
        message: impl Into<String>,
    ) -> Record {
        Record {
            timestamp: timestamp.into(),
            level,
            source: source.into(),
            message: message.into(),
        }
    }
}

/// 判定规则速查
///
/// 格式约定：
///
/// ```text
/// <timestamp> <LEVEL> [<source>] <message...>
/// ```
///
/// - R1 至少 4 个 token（按空白切分）
/// - R2 第 1 个 token 是 `YYYY-MM-DDTHH:MM:SSZ`（20 字符，月/日/时/分/秒在合理区间）
/// - R3 第 2 个 token 恰好是 `INFO` / `WARN` / `ERROR`
/// - R4 第 3 个 token 以 `[` 开头、以 `]` 结尾
/// - R5 第 3 个 token 之后必须有非空内容作为 message
/// - 违反任意一条 → 返回 `None`
///
/// 合法例子：
///
/// ```text
/// 2026-09-19T10:23:45Z INFO  [worker-3] task started id=1024
/// ```
///
/// 必须返回 `None` 的例子：
///
/// ```text
/// malformed line without structure
/// ```
///
/// # ⚠️ 一个必须先想清楚的问题：字段之间是**几个空格**？
///
/// 注意上面那个合法例子：`INFO` 后面是**两个空格**。这不是笔误。
/// 级别字段是 5 字符定宽对齐的（`INFO ` / `WARN ` / `ERROR`），
/// 所以 `INFO ` 和 `ERROR` 对齐之后，`INFO` 与 `[` 之间会出现两个空格。
/// 真实数据集里两种情况都有。
///
/// 这意味着一件很重要的事：**不要按单个空格切分。**
/// 以 `splitn(4, ' ')` 为例，对 `INFO  [worker-3]` 那行切分，第 3 段会变成
/// 空串，于是第 3 个 token 不以 `[` 开头、违反 R4，该行会被误判成脏行。
/// 注意这个错误并非每行都会犯：`ERROR [scheduler]` 只有一个空格，切分是对的。
/// 你需要自己决定怎么处理连续空白，并用测试证明你的选择是对的。
///
/// # 你要实现的部分
///
/// TODO(实验一): 实现这个函数，大约 25 行。
///
/// 提示方向（不给你实现）：
/// - 「字段之间可能有多个空格」和「message 内部含空格」这两件事同时存在时，
///   切分要怎么做？
/// - 剥掉方括号可以用 `strip_prefix` 和 `strip_suffix`。
/// - 时间戳要做一个格式校验，否则 `not a timestamp INFO [x] hi` 会被误收。
///
/// # 关于返回值里的 `to_string()`
///
/// 入参是 `&str`，而 [`Record`] 的字段是 `String`。所以你必然要做一次
/// 「从借用到拥有」的转换。这是你第一次在真实任务里碰到这个问题：
/// 什么时候可以继续借用，什么时候必须复制一份？想清楚再写。
pub fn parse_line(line: &str) -> Option<Record> {
    todo!("实验一：实现 parse_line")
}
