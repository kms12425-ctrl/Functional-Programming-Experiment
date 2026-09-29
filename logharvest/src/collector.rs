//! 实验三：并发采集与聚合
//!
//! 任务：实现 [`collect_concurrent`]，让多个日志文件被并发地处理。
//!
//! [`collect_sequential`] 是完整的单线程实现，已经给你了。它有两个作用：
//! 1. 正确性参照 —— 并发版的结果必须和它一致；
//! 2. 性能基线 —— `main.rs` 会把两者的耗时都打印出来。

use crate::extract::count_by_level;
use crate::parser::{parse_line, Level, Record};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

// ---------------------------------------------------------------------------
// AI 关卡：判断一个类型能不能跨线程
// ---------------------------------------------------------------------------

/// 「带本地缓存的采集配置」。
///
/// 它看起来很合理，实际上因为内部的 `Rc` 和 `RefCell` 而**既不是 `Send`
/// 也不是 `Sync`**。
///
/// # 你要做的事（详见《Rust 上机实验报告》实验三部分）
///
/// 1. 判断它是否实现 `Send`、是否实现 `Sync`，各写一句话理由。
/// 2. 写一段 8 行左右的代码，**让编译器给出结论**，把输出粘进实验报告。
/// 3. 把 [`LocalConfig`] 复制一份成 `SharedConfig`，把 `Rc` 换成 `Arc`、
///    `RefCell` 换成 `Mutex`，再验证一次，看结论怎么变。
/// 4. 用一句话解释：为什么 `Arc::clone` 不等于深拷贝。
///
/// **注意**：第 2 步和第 3 步你会撞上一个坑 —— `thread::spawn` 只检查
/// `Send`，它**不会**帮你验证 `Sync`。想知道一个类型是不是 `Sync`，
/// 你需要别的手段。这个坑是考查的一部分，报告里给了方向。
#[derive(Debug, Clone)]
pub struct LocalConfig {
    pub node_name: Rc<String>,
    pub cache: RefCell<Vec<String>>,
}

impl LocalConfig {
    /// 构造一个 `LocalConfig`。
    pub fn new(node_name: impl Into<String>) -> LocalConfig {
        LocalConfig {
            node_name: Rc::new(node_name.into()),
            cache: RefCell::new(Vec::new()),
        }
    }
}

// ---------------------------------------------------------------------------
// 采集结果
// ---------------------------------------------------------------------------

/// 一份采集报告的汇总数据。
///
/// 注意：这个类型要**跨线程传递**，所以它内部的每一个字段都必须是 `Send` 的。
/// 你在实现 `collect_concurrent` 时如果对它的设计有意见，可以改 —— 但改了之后
/// 要保证 `collect_sequential` 和测试集仍然编译通过。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Summary {
    /// 成功解析的记录总数
    pub total_records: usize,
    /// 被跳过的脏行数
    pub skipped_lines: usize,
    /// 每个文件的行数（按传入顺序）
    pub per_file_lines: Vec<usize>,
    /// 级别分布
    pub level_counts: HashMap<Level, usize>,
    /// 任务 ID 出现次数（只在能被解析成数字时统计）
    pub task_counts: HashMap<u64, usize>,
}

impl Summary {
    /// 构造一个空的汇总。你在实现 `collect_concurrent` 时会用到它。
    pub fn empty() -> Summary {
        Summary {
            total_records: 0,
            skipped_lines: 0,
            per_file_lines: Vec::new(),
            level_counts: HashMap::new(),
            task_counts: HashMap::new(),
        }
    }

    /// 把另一个汇总并入自身。用于把各线程的结果合并成最终报告。
    ///
    /// 注意它对 `per_file_lines` 是**追加**语义，所以调用顺序会影响结果。
    pub fn merge(&mut self, other: Summary) {
        self.total_records += other.total_records;
        self.skipped_lines += other.skipped_lines;
        self.per_file_lines.extend(other.per_file_lines);
        for (k, v) in other.level_counts {
            *self.level_counts.entry(k).or_insert(0) += v;
        }
        for (k, v) in other.task_counts {
            *self.task_counts.entry(k).or_insert(0) += v;
        }
    }

    /// 提取「高频任务 ID」列表，按出现次数降序，取前 `n` 个。
    ///
    /// 次数相同时按任务 ID 升序，保证输出稳定（测试集依赖这个稳定性）。
    pub fn top_tasks(&self, n: usize) -> Vec<(u64, usize)> {
        let mut v: Vec<(u64, usize)> = self.task_counts.iter().map(|(k, c)| (*k, *c)).collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        v.truncate(n);
        v
    }
}

/// 处理单个文件，返回它自己的那份 `Summary`。
///
/// 这是一个**普通函数**，不是闭包。它存在的意义是让你在写并发版本时
/// 有东西可以复用 —— 你不需要把读文件、解析、提取这一整套逻辑
/// 重新在闭包里写一遍。
///
/// 每个文件独立产生一份 `Summary`，这本身就说明了一个重要事实：
/// **线程之间不需要共享任何可变状态**。
pub fn process_file(path: &Path) -> Summary {
    let mut s = Summary::empty();
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => {
            // 读不到文件不算崩溃，记为 0 行。
            s.per_file_lines.push(0);
            return s;
        }
    };

    let mut lines = 0usize;
    let mut records: Vec<Record> = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        lines += 1;
        match parse_line(trimmed) {
            Some(rec) => records.push(rec),
            None => s.skipped_lines += 1,
        }
    }

    s.total_records = records.len();
    s.per_file_lines.push(lines);
    s.level_counts = count_by_level(&records);
    for rec in &records {
        if let Some(id) = crate::extract::extract_task_id(rec) {
            *s.task_counts.entry(id).or_insert(0) += 1;
        }
    }
    s
}

/// 单线程采集。**这个实现已经给你了，不需要改。**
///
/// 它同时是并发版的正确性参照和性能基线。
pub fn collect_sequential(paths: &[PathBuf]) -> Summary {
    let mut total = Summary::empty();
    for p in paths {
        total.merge(process_file(p));
    }
    total
}

/// 并发采集。**这是实验三唯一需要你实现的函数，大约 50 行。**
///
/// # 语义要求
///
/// - 结果必须与 [`collect_sequential`] 完全一致，包括 `per_file_lines`
///   的顺序（要按传入的 `paths` 顺序）。
/// - 每个文件由一个独立线程处理。
/// - 线程之间**不共享可变状态**，只通过 channel 把各自的结果送回主线程。
///
/// # 你要撞的三道坎
///
/// ## 坎一：`thread::spawn` 要求闭包满足 `'static`
///
/// 你的第一反应大概是把 `&path` 传进闭包（你在 C++ 里习惯传引用）。
/// 编译器会拒绝，理由是借用的生命周期无法保证长于线程。
///
/// 提示：想一想传进去的 `path` 是**借用**还是**拥有**。
///
/// ## 坎二：所有权在闭包边界上的转移
///
/// 把 `path` move 进闭包之后，如果你在闭包外还想再用 `path`，
/// 会得到 `use of moved value`。这道坎**不给提示**，因为它是所有权
/// 规则的直接应用。解决方案不止一种，都对。
///
/// ## 坎三（可能遇到）：跨线程数据的类型约束
///
/// 如果你想用某种统一的方式承载各线程的结果（比如把不同形状的结果
/// 装进一个 `Box<dyn ...>` 再发出去），编译器会告诉你它不能跨线程。
/// 这道坎连着 `Send`，也正是 AI 关卡要考的内容。**不给提示** ——
/// 如果你真的撞上了，说明你的设计比别人复杂。
///
/// # 返回值
///
/// 把各线程的 `Summary` 合并后返回。合并请用 [`Summary::merge`]，
/// 并注意 `per_file_lines` 的顺序要求。
///
/// # 提示
///
/// - 标准库的 `std::sync::mpsc` 提供了多生产者单消费者 channel。
///   **`Sender` 是可克隆的**（`mpsc` = multi-producer single-consumer），
///   每个线程需要自己的 `Sender`。
/// - 一个容易踩的坑：主线程手里那份 `Sender` 必须在收结果**之前**丢弃，
///   否则 `rx` 永远等不到通道关闭，程序会**死锁**。
///   死锁不报错，只是卡住 —— 如果 `cargo run` 卡住不动，先查这里。
/// - `thread::spawn` 返回 `JoinHandle`，你可以选择收集它，也可以选择
///   只靠 channel 收结果。
/// - [`process_file`] 可以直接在新线程里调用，它返回的 `Summary` 本身就是
///   可跨线程传递的（内部字段全是 `Send`）。所以你**不需要**用
///   `Box<dyn Trait>` 之类的东西去统一类型。
pub fn collect_concurrent(paths: &[PathBuf]) -> Summary {
    todo!("实验三：实现 collect_concurrent")
}

/// 分别测出单线程和并发两条路径的耗时，单位毫秒。
///
/// 这个函数已经给你了，`main.rs` 用它来打印加速比。
/// 它内部调用了 [`collect_sequential`] 和你实现的 [`collect_concurrent`]。
pub fn benchmark(paths: &[PathBuf]) -> (Summary, u128, Summary, u128) {
    let t0 = Instant::now();
    let seq = collect_sequential(paths);
    let seq_ms = t0.elapsed().as_millis();

    let t1 = Instant::now();
    let con = collect_concurrent(paths);
    let con_ms = t1.elapsed().as_millis();

    (seq, seq_ms, con, con_ms)
}

/// 一个供关卡使用的占位函数，用来说明「`Arc` 版本的配置」应该长什么样。
///
/// 你不必实现它，但如果你在关卡第 3 步里想有一个现成的类型可以对照，
/// 可以参考它的字段布局。
#[allow(dead_code)]
pub struct SharedConfigSketch {
    pub node_name: Arc<String>,
    pub cache: std::sync::Mutex<Vec<String>>,
}

/// 供测试与关卡使用：断言一个类型是 `Send`。
///
/// 用法：`assert_send::<LocalConfig>();` —— 编译不过就说明不是 `Send`。
///
/// 注意它**只能验证 `Send`，不能验证 `Sync`**。这是有意的：
/// 关卡第 2 步就是想让你自己发现，`Send` 和 `Sync` 需要两套不同的验证手段。
pub fn assert_send<T: Send>() {}
