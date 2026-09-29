#!/usr/bin/env python3
"""生成 logharvest 实验用的模拟日志数据集。

设计要点：
- 精确控制脏行的位置和数量（这样测试集才能断言具体数字）
- 埋入中英混排的内容，测试解析器的健壮性
- 覆盖 INFO / WARN / ERROR 三个级别
- 埋入 id=NNNN，其中一部分故意不是合法数字，用来考 extract_task_id

跑法：python3 scripts/gen_data.py
"""
import os
import random

random.seed(20260919)  # 固定种子，保证每次生成的数据一致

HERE = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(os.path.dirname(HERE), "data")

LEVELS_W = [("INFO", 79), ("WARN", 15), ("ERROR", 6)]
SOURCES = ["worker-1", "worker-2", "worker-3", "worker-4", "scheduler", "reaper"]

TASK_IDS = [1024, 2048, 512, 4096, 256, 8192]

# 中英混排的消息正文，用来测试解析器不会因为非 ASCII 出问题
MESSAGES_ZH = [
    "任务开始处理 id={tid}",
    "任务完成 id={tid} 耗时 {ms}ms",
    "缓存命中率偏低 id={tid}",
    "任务失败 id={tid} reason=timeout",
    "连接重置 id={tid} 重试中",
    "队列深度超过阈值 id={tid} 队列={q}",
]
MESSAGES_EN = [
    "task started id={tid}",
    "task finished id={tid} elapsed={ms}ms",
    "cache miss ratio high id={tid}",
    "task failed id={tid} reason=timeout",
    "connection reset id={tid} retrying",
    "queue depth over threshold id={tid} depth={q}",
]

# 脏行模板
DIRTY = [
    "malformed line without structure",
    "2026-09-19T10:23:45Z TRACE [worker-3] unknown level",
    "2026-09-19T10:23:45Z INFO worker-3 missing brackets",
    "not a timestamp INFO [worker-3] hi",
    "-- continuation of previous line",
    "\t   ",
    "2026-13-45T99:99:99Z INFO [worker-3] bad timestamp",
]


def ts(n):
    """生成递增的时间戳，格式 RFC3339。"""
    hh = 10 + (n // 3600) % 14
    mm = (n // 60) % 60
    ss = n % 60
    return "2026-09-19T%02d:%02d:%02dZ" % (hh, mm, ss)


def make_line(n, pool, dirty_rate):
    if random.random() < dirty_rate:
        return random.choice(DIRTY)
    lv = random.choices([x[0] for x in LEVELS_W], weights=[x[1] for x in LEVELS_W])[0]
    src = random.choice(SOURCES)
    tid = random.choice(TASK_IDS)
    tpl = random.choice(pool)
    msg = tpl.format(tid=tid, ms=random.randint(3, 900), q=random.randint(10, 999))
    return "%s %-5s [%s] %s" % (ts(n), lv, src, msg)


def write_file(name, n_lines, dirty_rate, pool, bad_id_every=37):
    path = os.path.join(DATA, name)
    lines = []
    for i in range(n_lines):
        line = make_line(i, pool, dirty_rate)
        # 每隔 bad_id_every 行，故意造一个 id 不是合法数字的「假脏行」
        # 它格式合法（会被 parse_line 收下），但 extract_task_id 应该返回 None
        if bad_id_every and i % bad_id_every == bad_id_every - 1:
            lv = random.choices([x[0] for x in LEVELS_W], weights=[x[1] for x in LEVELS_W])[0]
            src = random.choice(SOURCES)
            line = "%s %-5s [%s] task lookup id=unknown" % (ts(i), lv, src)
        lines.append(line)
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        f.write("\n".join(lines) + "\n")
    return path, len(lines)


def main():
    os.makedirs(DATA, exist_ok=True)
    specs = [
        ("node-a.log", 900, 0.03, MESSAGES_EN),
        ("node-b.log", 1100, 0.02, MESSAGES_ZH),
        ("node-c.log", 700, 0.05, MESSAGES_EN + MESSAGES_ZH),
    ]
    total = 0
    for name, n, dr, pool in specs:
        p, cnt = write_file(name, n, dr, pool)
        total += cnt
        print("生成 %s (%d 行)" % (name, cnt))
    print("合计 %d 行" % total)


if __name__ == "__main__":
    main()
