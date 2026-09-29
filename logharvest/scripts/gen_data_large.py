#!/usr/bin/env python3
"""生成「加压版」数据集，用于让并发加速比可测量。

小数据集（2700 行）下 I/O 时间与线程启动开销同量级，加速比只有 1.5x，
无法支撑「性能对比」这道题。这个脚本按倍数放大数据量。

用法：
    python3 scripts/gen_data_large.py [倍数]   # 默认 20 倍，约 54000 行
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
DATA = os.path.join(ROOT, "data")
OUT = os.path.join(ROOT, "data_large")


def main():
    mult = int(sys.argv[1]) if len(sys.argv) > 1 else 20
    os.makedirs(OUT, exist_ok=True)
    total = 0
    for name in ["node-a.log", "node-b.log", "node-c.log"]:
        src = os.path.join(DATA, name)
        with open(src, encoding="utf-8") as f:
            block = f.read()
        # 简单重复：注意时间戳会重复，但解析器不校验唯一性，不影响实验
        body = "".join(block for _ in range(mult))
        dst = os.path.join(OUT, name)
        with open(dst, "w", encoding="utf-8", newline="\n") as f:
            f.write(body)
        n = len(body.splitlines())
        total += n
        print("生成 %s (%d 行, %.1f MB)" % (name, n, os.path.getsize(dst) / 1048576))
    print("合计 %d 行" % total)


if __name__ == "__main__":
    main()
