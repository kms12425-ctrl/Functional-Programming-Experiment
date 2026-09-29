//! logharvest —— 并发日志采集分析器
//!
//! 这是实验的库入口。三个实验分别产出三个模块：
//!
//! | 实验 | 模块 | 职责 |
//! |---|---|---|
//! | 一 | [`parser`] | 把一行原始日志文本变成结构化的 [`Record`] |
//! | 二 | [`extract`] | 从 [`Record`] 里取出有价值的字段并做初步统计 |
//! | 三 | [`collector`] | 并发地采集多个文件，汇总成一份报告 |
//!
//! [`Record`]: parser::Record

pub mod collector;
pub mod extract;
pub mod parser;

pub use collector::{collect_concurrent, collect_sequential, Summary};
pub use extract::{count_by_level, extract_task_id, find_field};
pub use parser::{parse_line, Level, Record};
