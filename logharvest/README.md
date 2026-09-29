# logharvest —— 并发日志采集分析器

**Rust 上机实验代码包**

```
$ cargo run --release -- data/node-a.log data/node-b.log data/node-c.log

logharvest v0.1 — 并发日志采集分析器
========================================
采集文件: 3 个
总记录数: 2624 (跳过脏行: 67)
采集耗时: 2 ms (并发) / 3 ms (单线程, 1.5x 加速)

级别分布:
  INFO      2067  (78.8%)
  WARN       387  (14.7%)
  ERROR      170  (6.5%)

涉及任务: 6 个
高频任务 ID: #8192 (448 次), #512 (434 次), #256 (429 次)
```

---

## 这个包里有什么

```
logharvest/
├── Cargo.toml          零依赖；release 保留 debug 信息以缩短编译
├── src/
│   ├── lib.rs          模块声明与 re-export
│   ├── parser.rs       实验一：parse_line（留空，待补）
│   ├── extract.rs      实验二：extract_task_id / count_by_level（留空，待补）
│   │                   find_field 已给
│   ├── collector.rs    实验三：collect_concurrent（留空，待补）
│   │                   collect_sequential / process_file / Summary 已给
│   │                   LocalConfig（AI 关卡素材）、assert_send 已给
│   └── main.rs         命令行入口与报告输出（已给，不需改）
├── tests/
│   ├── lab01_parser.rs      11 条
│   ├── lab02_extract.rs     11 条
│   └── lab03_concurrent.rs  10 条
├── data/               固定种子生成：node-a/b/c.log，共 2700 行
└── scripts/
    ├── gen_data.py         重新生成数据集
    ├── gen_data_large.py   生成加压数据集（测加速比用）
    └── checkpoint_code.rs  AI 关卡辅助代码
```

**所有实验说明、题目要求、简答题、提交要求都在实验报告模板里，
不在本代码包中。** 请对照实验报告完成。

## 快速开始

```bash
cargo test                    # 跑全部测试（初始状态会因 todo!() 而失败，正常）
cargo test --test lab01_parser
cargo run --release -- data/node-a.log data/node-b.log data/node-c.log
```

**不要修改 `tests/` 目录下的任何文件。** 它们是评分依据。

## 关于本工程的几个事实

### 零第三方依赖

`Cargo.toml` 的 `[dependencies]` 是空的，而且**必须保持为空**。

原因有两个。一是环境问题：一个引入依赖的工程，第一次 `cargo build` 要联网拉
crate，网络抖动、镜像不一致、代理拦截，任何一条都会让人卡在编译之前，
而这段时间不产出任何学习效果。

更隐蔽的问题是**依赖会掩盖语言特性**。引入 `regex`，实验一就变成"查 regex
API 怎么写"；引入 `serde`，实验二就交给 derive 宏，你碰不到 `match` 和迭代器。
这两个实验本来就是要练这些机制的。

标准库足够完成全部四个函数。

### 数据集

`data/` 下三个文件由 `scripts/gen_data.py` 生成（固定随机种子，可复现）：

| 文件 | 总行 | 非空行 |
|---|---|---|
| `node-a.log` | 900 | 897 |
| `node-b.log` | 1100 | 1099 |
| `node-c.log` | 700 | 695 |
| **合计** | **2700** | **2691** |

数据里刻意包含：**中英混排**的消息正文、**空行**、**格式错乱**的脏行、
**格式合法但 `id` 不是数字**的行。

### 关于性能对比的一个诚实说明

你可能发现小数据集上的加速比只有 1.5x 左右。这是正常的，原因有两条：

1. **只有 3 个文件，最多 3 个线程**，理论上限就是 3x。
2. 处理 2700 行的总耗时只有 2–3 毫秒，**线程启动开销与 I/O 时间同量级**。

想看到更稳定的加速比，用加压版数据：

```bash
python3 scripts/gen_data_large.py 20
cargo run --release -- data_large/node-a.log data_large/node-b.log data_large/node-c.log
```

在 12 核机器上，20 倍数据能稳定跑出 **1.9x–2.6x**，60 倍数据依旧在这个区间 ——
**不会再高了，因为文件数决定了线程数的上限**。这个现象本身是实验报告里要你解释的。

`Cargo.toml` 里开了 `[profile.release] debug = true`，是为了让 release 模式的
编译不至于太慢。**测性能一定要用 `--release`**，debug 模式的数字没有意义。

### AI 关卡辅助代码

`scripts/checkpoint_code.rs` 给的是 `Send` / `Sync` 的**验证手段示例**，
不是答案。它演示了三种办法：`thread::spawn`（只测 `Send`）、
`thread::scope` 共享 `&T`（测 `Sync`）、以及编译期断言。
具体怎么用，实验报告里 AI 关卡那一节会讲。

## 常见卡点

| 现象 | 原因 |
|---|---|
| `cargo test` 一开始全红 | 正常，`todo!()` 会 panic |
| `argument requires that ... be valid for 'static` | 见实验报告「三道坎」坎一 |
| `use of moved value` | 见实验报告「三道坎」坎二 |
| `cargo run` 卡住不动 | 主线程的 `Sender` 没 drop，死锁 |
| `per_file_lines` 顺序不对 | 见实验报告「顺序语义」 |
| 大部分正常行被判成脏行 | 看看 `parse_line` 是怎么切分空格的 |
