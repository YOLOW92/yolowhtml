#!/usr/bin/env python3
"""两个版本的输出做配对统计比较：质量差异的置信区间和显著性，以及耗时比。

    python3 -I benchmarks/quality/compare.py <aeb|wcxb|wmb> <old.jsonl> <new.jsonl> [--by-type]

同一页在新旧两个版本上的分数是成对的，所以用配对方法，不比两组的平均数：

- 每页 F1 的差 d_i = F1_new(i) − F1_old(i)。
- **配对 bootstrap**（Efron & Tibshirani 1993）：有放回地重抽页面 B 次，每次算平均差，取
  2.5% / 97.5% 分位数作为 95% 置信区间。不假设分数服从正态分布。
- **配对随机化检验**（Fisher–Pitman 的符号翻转检验）：原假设是"新旧版本可以互换"，
  那么每个 d_i 的正负号都等可能；随机翻转符号 B 次，看平均差的绝对值不小于观测值的比例，
  得到双侧 p 值。全为 0 的差（输出一样）不影响结果。
- **耗时**：每页耗时比 t_new / t_old 的几何平均（对数域平均再取指数，几个特别慢的页不会
  主导结果），同样给 bootstrap 区间。

页面分数：WCXB、WebMainBench 用各自评分脚本里的词级 F1；AEB 用它官方 evaluate.py 的 4-gram
shingle F1（逐页 tp/fp/fn 归一化后算）。只用标准库。
"""

from __future__ import annotations

import json
import math
import random
import statistics
import sys
from collections import defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent
DATA = HERE.parent / "data"
RUNS = DATA / "runs"
sys.path.insert(0, str(HERE))

B = 10_000


def load(path: Path) -> dict[str, dict]:
    return {str(r["id"]): r for r in map(json.loads, path.read_text().splitlines()) if r}


def scorer(kind: str):
    """返回 (gold: id → 参考文本, page_type: id → 类型, f1(markdown, gold) -> float)。"""
    if kind == "wcxb":
        import wcxb

        man = {r["id"]: r for r in map(json.loads, (RUNS / "wcxb-in" / "manifest.jsonl").read_text().splitlines())}
        gold, types = {}, {}
        for k, r in man.items():
            text, _, ptype = wcxb.get_gt_info(DATA / "wcxb" / r["split"] / "ground-truth" / f"{k}.json")
            gold[k], types[k] = text, ptype or "?"
        return gold, types, lambda md, g: wcxb.word_f1(wcxb.plain(md), g)[2]
    if kind == "wmb":
        import webmain

        gold, types = {}, {}
        for r in map(json.loads, (RUNS / "wmb-in" / "gold.jsonl").read_text().splitlines()):
            gold[str(r["id"])] = r.get("groundtruth", "")
            types[str(r["id"])] = r.get("layout_type") or r.get("type") or "?"
        return gold, types, lambda md, g: webmain.word_f1(webmain.plain(md), g)[2]
    if kind == "aeb":
        import aeb

        # AEB 官方评分（已审阅：纯计算，无 I/O 副作用）
        sys.path.insert(0, str(DATA / "aeb"))
        import evaluate as aeb_eval  # noqa: E402

        truth = json.loads((DATA / "aeb" / "ground-truth.json").read_text())
        gold = {k: v.get("articleBody", "") for k, v in truth.items()}

        def f1(md: str, g: str) -> float:
            tp, fp, fn = aeb_eval.string_shingle_matching(true=g, pred=aeb.plain(md))
            p = aeb_eval.precision_score(tp, fp, fn)
            r = aeb_eval.recall_score(tp, fp, fn)
            return 0.0 if p + r == 0 else 2 * p * r / (p + r)

        return gold, {k: "article" for k in gold}, f1
    raise SystemExit(f"未知数据集 {kind}")


def bootstrap_ci(values: list[float], stat, rng: random.Random) -> tuple[float, float]:
    n = len(values)
    samples = sorted(stat([values[rng.randrange(n)] for _ in range(n)]) for _ in range(B))
    return samples[int(0.025 * B)], samples[int(0.975 * B) - 1]


def sign_flip_p(diffs: list[float], rng: random.Random) -> float:
    nz = [d for d in diffs if d != 0]
    if not nz:
        return 1.0
    obs = abs(sum(nz))
    hits = sum(abs(sum(d if rng.random() < 0.5 else -d for d in nz)) >= obs - 1e-12 for _ in range(B))
    return (hits + 1) / (B + 1)


def report(name: str, diffs: list[float], old: list[float], new: list[float], rng: random.Random) -> str:
    lo, hi = bootstrap_ci(diffs, statistics.fmean, rng)
    p = sign_flip_p(diffs, rng)
    changed = sum(d != 0 for d in diffs)
    better = sum(d > 0 for d in diffs)
    return (
        f"{name:<16} n={len(diffs):<5} F1 {statistics.fmean(old):.4f} → {statistics.fmean(new):.4f}  "
        f"Δ={statistics.fmean(diffs):+.4f} [95% CI {lo:+.4f}, {hi:+.4f}]  p={p:.3g}  "
        f"变化 {changed} 页（变好 {better}）"
    )


def main() -> None:
    kind, old_path, new_path = sys.argv[1], Path(sys.argv[2]), Path(sys.argv[3])
    by_type = "--by-type" in sys.argv
    rng = random.Random(20261117)
    gold, types, f1 = scorer(kind)
    old, new = load(old_path), load(new_path)
    ids = [k for k in old if k in new and k in gold]
    fo = {k: f1(old[k]["markdown"], gold[k]) for k in ids}
    fn = {k: f1(new[k]["markdown"], gold[k]) if new[k]["markdown"] != old[k]["markdown"] else fo[k] for k in ids}
    groups: dict[str, list[str]] = {"全部": ids}
    if by_type:
        for k in ids:
            groups.setdefault(types[k], []).append(k)
    for name, ks in groups.items():
        print(report(name, [fn[k] - fo[k] for k in ks], [fo[k] for k in ks], [fn[k] for k in ks], rng))
    ratios = [
        math.log(new[k]["ms"] / old[k]["ms"])
        for k in ids
        if old[k].get("ms") and new[k].get("ms") and old[k]["ms"] > 0 and new[k]["ms"] > 0
    ]
    if ratios:
        gm = math.exp(statistics.fmean(ratios))
        lo, hi = bootstrap_ci(ratios, statistics.fmean, rng)
        print(f"{'耗时比（几何平均）':<14} {gm:.3f} [95% CI {math.exp(lo):.3f}, {math.exp(hi):.3f}]（<1 表示新版更快）")


if __name__ == "__main__":
    main()
