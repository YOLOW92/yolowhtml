#!/usr/bin/env python3
"""Generate SVG benchmark charts and architecture diagrams for yolowhtml 0.1.0."""

from __future__ import annotations

from html import escape
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "docs" / "figures"

FONT_SANS = "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif"
FONT_MONO = "ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, monospace"

BG = "#f8fafc"
CARD = "#ffffff"
BORDER = "#e2e8f0"
TEXT = "#0f172a"
SUB = "#475569"
MUTED = "#64748b"

NAVY = "#0f172a"
NAVY_2 = "#1e293b"
INDIGO = "#4f46e5"
INDIGO_SOFT = "#eef2ff"
INDIGO_BORDER = "#c7d2fe"
TEAL = "#0d9488"
TEAL_SOFT = "#f0fdfa"
TEAL_BORDER = "#99f6e4"
BLUE = "#0284c7"
BLUE_SOFT = "#f0f9ff"
BLUE_BORDER = "#bae6fd"
AMBER = "#d97706"
AMBER_SOFT = "#fffbeb"
AMBER_BORDER = "#fde68a"
EMERALD = "#059669"
EMERALD_SOFT = "#ecfdf5"
EMERALD_BORDER = "#a7f3d0"
SLATE_BAR = "#94a3b8"


def svg_doc(width: int, height: int, body: list[str]) -> str:
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" '
        f'width="100%" role="img">\n'
        "  <defs>\n"
        '    <linearGradient id="headerGrad" x1="0" y1="0" x2="1" y2="0">\n'
        f'      <stop offset="0%" stop-color="{NAVY}"/>\n'
        f'      <stop offset="60%" stop-color="{NAVY_2}"/>\n'
        '      <stop offset="100%" stop-color="#312e81"/>\n'
        "    </linearGradient>\n"
        '    <linearGradient id="primaryGrad" x1="0" y1="0" x2="1" y2="0">\n'
        '      <stop offset="0%" stop-color="#4338ca"/>\n'
        '      <stop offset="100%" stop-color="#6366f1"/>\n'
        "    </linearGradient>\n"
        '    <linearGradient id="tealGrad" x1="0" y1="0" x2="1" y2="0">\n'
        '      <stop offset="0%" stop-color="#0f766e"/>\n'
        '      <stop offset="100%" stop-color="#14b8a6"/>\n'
        "    </linearGradient>\n"
        '    <filter id="cardShadow" x="-4%" y="-4%" width="108%" height="112%">\n'
        '      <feDropShadow dx="0" dy="2" stdDeviation="3" flood-color="#0f172a" flood-opacity="0.05"/>\n'
        "    </filter>\n"
        '    <marker id="arrow" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">\n'
        f'      <path d="M 0 1.5 L 8 5 L 0 8.5 z" fill="{INDIGO}"/>\n'
        "    </marker>\n"
        "  </defs>\n"
        "  <style>\n"
        f"    .title {{ font: 700 18.5px {FONT_SANS}; fill: #ffffff; letter-spacing: -0.2px; }}\n"
        f"    .subtitle {{ font: 400 12px {FONT_SANS}; fill: #cbd5e1; }}\n"
        f"    .panel-title {{ font: 700 14.5px {FONT_SANS}; fill: {TEXT}; }}\n"
        f"    .panel-sub {{ font: 400 11.5px {FONT_SANS}; fill: {MUTED}; }}\n"
        f"    .label {{ font: 500 12px {FONT_SANS}; fill: {SUB}; }}\n"
        f"    .label-hi {{ font: 700 12.5px {FONT_SANS}; fill: {TEXT}; }}\n"
        f"    .val {{ font: 500 11.5px {FONT_MONO}; fill: {SUB}; }}\n"
        f"    .val-hi {{ font: 700 12px {FONT_MONO}; fill: {INDIGO}; }}\n"
        f"    .val-teal {{ font: 700 12px {FONT_MONO}; fill: {TEAL}; }}\n"
        f"    .kpi-num {{ font: 800 22px {FONT_MONO}; letter-spacing: -0.5px; }}\n"
        f"    .kpi-title {{ font: 700 11px {FONT_SANS}; fill: {TEXT}; }}\n"
        f"    .kpi-sub {{ font: 400 10.5px {FONT_SANS}; fill: {MUTED}; }}\n"
        f"    .badge {{ font: 700 10.5px {FONT_MONO}; }}\n"
        f"    .code {{ font: 600 11.5px {FONT_MONO}; fill: {TEXT}; }}\n"
        f"    .note {{ font: 400 11px {FONT_SANS}; fill: {SUB}; }}\n"
        "  </style>\n"
        f'  <rect width="{width}" height="{height}" rx="14" fill="{BG}" stroke="{BORDER}"/>\n'
        + "\n".join(body)
        + "\n</svg>\n"
    )


def header_band(width: int, title: str, subtitle: str, badge_text: str) -> list[str]:
    bw = max(len(badge_text) * 7 + 26, 130)
    bx = width - bw - 22
    return [
        f'  <rect x="0" y="0" width="{width}" height="68" rx="14" fill="url(#headerGrad)"/>',
        f'  <rect x="0" y="54" width="{width}" height="14" fill="url(#headerGrad)"/>',
        f'  <text x="26" y="30" class="title">{escape(title)}</text>',
        f'  <text x="26" y="50" class="subtitle">{escape(subtitle)}</text>',
        f'  <rect x="{bx}" y="19" width="{bw}" height="28" rx="14" fill="#1e1b4b" stroke="#6366f1" stroke-width="1"/>',
        f'  <text x="{bx + bw / 2:.1f}" y="37" text-anchor="middle" class="badge" fill="#e0e7ff">{escape(badge_text)}</text>',
    ]


def card_box(x: float, y: float, w: float, h: float, title: str = "", sub: str = "") -> list[str]:
    out = [
        f'  <rect x="{x}" y="{y}" width="{w}" height="{h}" rx="10" fill="{CARD}" stroke="{BORDER}" filter="url(#cardShadow)"/>',
    ]
    if title:
        out.append(f'  <text x="{x + 18}" y="{y + 25}" class="panel-title">{escape(title)}</text>')
    if sub:
        out.append(f'  <text x="{x + 18}" y="{y + 43}" class="panel-sub">{escape(sub)}</text>')
    return out


def make_hero_html_bench() -> None:
    width, height = 1040, 440
    body = header_band(
        width,
        "yolowhtml — HTML to Markdown & Main-Content Extraction Benchmark",
        "i7-10700 · 15 GB RAM · Zero-copy arena DOM + single-pass content & metadata extraction",
        "0.24 ms/page · 1.000 F1",
    )

    kpis = [
        ("0.24 ms/page", "HTML → Markdown Speed", "8.12x faster than Turndown (1.9 ms)", INDIGO, INDIGO_SOFT, INDIGO_BORDER),
        ("1.0 ms/page", "Main Content (597 pages)", "7.26x faster than crw (7.4 ms)", TEAL, TEAL_SOFT, TEAL_BORDER),
        ("1.000 F1", "Markdown Structure Fidelity", "600 docs · 15 categories · vs 0.936", EMERALD, EMERALD_SOFT, EMERALD_BORDER),
        ("20 MB Peak", "Per-Worker Memory (A2)", "60.8x less than Readability+Turndown", BLUE, BLUE_SOFT, BLUE_BORDER),
    ]
    kw = (width - 40 - 36) / 4
    for i, (num, t, s, col, bg, bd) in enumerate(kpis):
        kx = 20 + i * (kw + 12)
        ky = 82
        body.extend([
            f'  <rect x="{kx:.1f}" y="{ky}" width="{kw:.1f}" height="76" rx="10" fill="{CARD}" stroke="{bd}" filter="url(#cardShadow)"/>',
            f'  <rect x="{kx:.1f}" y="{ky}" width="6" height="76" rx="3" fill="{col}"/>',
            f'  <text x="{kx + 18:.1f}" y="{ky + 29}" class="kpi-num" fill="{col}">{escape(num)}</text>',
            f'  <text x="{kx + 18:.1f}" y="{ky + 48}" class="kpi-title">{escape(t)}</text>',
            f'  <text x="{kx + 18:.1f}" y="{ky + 64}" class="kpi-sub">{escape(s)}</text>',
        ])

    # Left panel: Main-content extraction latency (p50 ms/page ↓)
    pw = (width - 54) / 2
    py, ph = 172, 248
    body.extend(card_box(20, py, pw, ph, "Main-Content Extraction Speed (p50 ms/page ↓)", "Firecrawl 597-page corpus · Single thread · Lower is faster"))

    rows_left = [
        ("yolowhtml", 1.0, "1.0 ms/page  (7.26x faster · 6,489 pages/s @ 16T)", True),
        ("crw (Rust)", 7.4, "7.4 ms/page  (p95 36.0 ms)", False),
        ("trafilatura", 21.2, "21.2 ms/page  (p95 136 ms)", False),
        ("Crawl4AI 0.9", 38.6, "38.6 ms/page  (p95 157 ms)", False),
        ("MarkItDown", 40.4, "40.4 ms/page  (p95 145 ms)", False),
        ("Readability+TD", 174.0, "174.0 ms/page  (p95 1,170 ms)", False),
    ]
    bx = 128
    bw = pw - 128 - 265
    for idx, (name, val, label, hi) in enumerate(rows_left):
        ry = py + 62 + idx * 29
        if hi:
            body.append(f'  <rect x="28" y="{ry - 4}" width="{pw - 16:.1f}" height="24" rx="5" fill="{INDIGO_SOFT}"/>')
        body.append(
            f'  <text x="{bx - 10}" y="{ry + 12}" text-anchor="end" class="{"label-hi" if hi else "label"}">{escape(name)}</text>'
        )
        # Visual bar proportional to speed (1 / val^0.45) or latency? Let's plot latency with power scale so lower bar = faster!
        w = max((val / 174.0) ** 0.45 * bw, 8.0)
        fill = "url(#primaryGrad)" if hi else SLATE_BAR
        body.append(f'  <rect x="{bx}" y="{ry}" width="{w:.1f}" height="16" rx="4" fill="{fill}"/>')
        body.append(
            f'  <text x="{bx + w + 8:.1f}" y="{ry + 12}" class="{"val-hi" if hi else "val"}">{escape(label)}</text>'
        )

    # Right panel: HTML -> Markdown & Date Extraction Speed (ms/page ↓)
    rx = 20 + pw + 14
    body.extend(card_box(rx, py, pw, ph, "HTML → Markdown & Date Speed (ms/page ↓)", "600 generated docs & 1,000 htmldate pages · Lower is faster"))

    rows_right = [
        ("yolowhtml (MD)", 0.24, "0.24 ms/page  (8.12x faster · F1 1.000)", True),
        ("Turndown (MD)", 1.95, "1.95 ms/page  (F1 0.799)", False),
        ("html2text (MD)", 2.10, "2.10 ms/page  (F1 0.852)", False),
        ("markdownify (MD)", 5.40, "5.40 ms/page  (F1 0.936)", False),
        ("yolowhtml (Date)", 0.31, "0.31 ms/page  (6.45x faster · F1 0.945)", True),
        ("htmldate (Date)", 4.04, "4.04 ms/page  (extensive · F1 0.945)", False),
    ]
    bx2 = rx + 132
    bw2 = pw - 132 - 235
    for idx, (name, val, label, hi) in enumerate(rows_right):
        ry = py + 62 + idx * 29
        if hi:
            body.append(f'  <rect x="{rx + 8:.1f}" y="{ry - 4}" width="{pw - 16:.1f}" height="24" rx="5" fill="{TEAL_SOFT}"/>')
        body.append(
            f'  <text x="{bx2 - 10:.1f}" y="{ry + 12}" text-anchor="end" class="{"label-hi" if hi else "label"}">{escape(name)}</text>'
        )
        w = max((val / 5.40) ** 0.55 * bw2, 8.0)
        fill = "url(#tealGrad)" if hi else SLATE_BAR
        body.append(f'  <rect x="{bx2:.1f}" y="{ry}" width="{w:.1f}" height="16" rx="4" fill="{fill}"/>')
        body.append(
            f'  <text x="{bx2 + w + 8:.1f}" y="{ry + 12}" class="{"val-teal" if hi else "val"}">{escape(label)}</text>'
        )

    (OUT / "hero-html-bench.svg").write_text(svg_doc(width, height, body), encoding="utf-8")


def make_quality_f1_bench() -> None:
    width, height = 1040, 360
    body = header_band(
        width,
        "yolowhtml — Extraction Quality, Markdown Fidelity & Publication Date Accuracy",
        "Evaluated across 600 Markdown fidelity docs, AEB (150), WCXB (150), htmldate (1,000), and 166 hard web pages",
        "Top Quality Across 5 Suites",
    )

    pw = (width - 54) / 2
    py, ph = 84, 256
    body.extend(card_box(20, py, pw, ph, "Main Content, Dates & Hard Pages (F1 / Completion ↑)", "yolowhtml vs best competing tool on each benchmark"))

    left_rows = [
        ("AEB News (150p)", 0.954, "0.954 F1  (vs trafilatura 0.950)", "primary"),
        ("htmldate (1,000p)", 0.945, "0.945 F1  (ties htmldate @ 13x)", "teal"),
        ("WCXB Test (150p)", 0.866, "0.866 F1  (vs Readability 0.848)", "primary"),
        ("WebMainBench", 0.833, "0.833 F1  (ties trafilatura 0.833)", "teal"),
        ("Hard Pages (166p)", 0.723, "0.723 done  (vs crw 0.590)", "primary"),
    ]
    bx = 148
    bw = pw - 148 - 225
    for idx, (name, sc, label, kind) in enumerate(left_rows):
        ry = py + 66 + idx * 35
        if kind == "primary":
            body.append(f'  <rect x="28" y="{ry - 5}" width="{pw - 16:.1f}" height="26" rx="5" fill="{INDIGO_SOFT}"/>')
        body.append(
            f'  <text x="{bx - 10}" y="{ry + 12}" text-anchor="end" class="label-hi">{escape(name)}</text>'
        )
        w = max((sc / 1.0) * bw, 5.0)
        fill = "url(#primaryGrad)" if kind == "primary" else "url(#tealGrad)"
        cls = "val-hi" if kind == "primary" else "val-teal"
        body.append(f'  <rect x="{bx}" y="{ry}" width="{w:.1f}" height="16" rx="4" fill="{fill}"/>')
        body.append(f'  <text x="{bx + w + 8:.1f}" y="{ry + 12}" class="{cls}">{escape(label)}</text>')

    rx = 20 + pw + 14
    body.extend(card_box(rx, py, pw, ph, "HTML → Markdown Structural Fidelity (Mean F1 ↑)", "600 generated docs · Tables, Code, Lists, Math, Escaping"))

    right_rows = [
        ("yolowhtml", 1.000, "1.000 F1  (Tables 1.00 · Math 1.00)", "primary"),
        ("markdownify", 0.936, "0.936 F1  (Tables 0.83 · Math 0.34)", "other"),
        ("Defuddle", 0.912, "0.912 F1  (Lists 0.54 · Math 0.57)", "other"),
        ("html2text", 0.852, "0.852 F1  (Code 0.11 · Math 0.25)", "other"),
        ("Turndown", 0.799, "0.799 F1  (Tables 0.00 · Math 0.17)", "other"),
    ]
    bx2 = rx + 114
    bw2 = pw - 114 - 252
    for idx, (name, sc, label, kind) in enumerate(right_rows):
        ry = py + 66 + idx * 35
        hi = kind == "primary"
        if hi:
            body.append(f'  <rect x="{rx + 8:.1f}" y="{ry - 5}" width="{pw - 16:.1f}" height="26" rx="5" fill="{TEAL_SOFT}"/>')
        body.append(
            f'  <text x="{bx2 - 10:.1f}" y="{ry + 12}" text-anchor="end" class="{"label-hi" if hi else "label"}">{escape(name)}</text>'
        )
        w = max((sc / 1.0) * bw2, 5.0)
        fill = "url(#tealGrad)" if hi else SLATE_BAR
        cls = "val-teal" if hi else "val"
        body.append(f'  <rect x="{bx2:.1f}" y="{ry}" width="{w:.1f}" height="16" rx="4" fill="{fill}"/>')
        body.append(f'  <text x="{bx2 + w + 8:.1f}" y="{ry + 12}" class="{cls}">{escape(label)}</text>')

    (OUT / "quality-f1-bench.svg").write_text(svg_doc(width, height, body), encoding="utf-8")


def make_memory_bench() -> None:
    width, height = 1040, 280
    body = header_band(
        width,
        "Peak Worker Memory & Arena Reuse — A2 HTML Main-Content Corpus",
        "Zero-copy source slicing + contiguous Vec<Node> arena + ProcessWorkspace recycling across pages",
        "20 MB Peak · #![forbid(unsafe_code)]",
    )

    pw = (width - 54) / 2
    py, ph = 84, 178
    body.extend(card_box(20, py, pw, ph, "Peak Memory per Worker (MB ↓)", "A2 real-web HTML pages · Lower memory is better"))

    rows = [
        ("yolowhtml", 20.0, "20 MB peak  (6.2x less than trafilatura)", True),
        ("trafilatura", 123.0, "123 MB peak  (Python + lxml DOM)", False),
        ("Readability+TD", 1200.0, "1,200 MB peak  (60.8x more memory)", False),
    ]
    bx = 126
    bw = pw - 126 - 235
    for idx, (name, val, label, hi) in enumerate(rows):
        ry = py + 62 + idx * 34
        if hi:
            body.append(f'  <rect x="28" y="{ry - 4}" width="{pw - 16:.1f}" height="24" rx="5" fill="{INDIGO_SOFT}"/>')
        body.append(f'  <text x="{bx - 10}" y="{ry + 12}" text-anchor="end" class="{"label-hi" if hi else "label"}">{escape(name)}</text>')
        w = max((val / 1200.0) ** 0.5 * bw, 10.0)
        fill = "url(#primaryGrad)" if hi else SLATE_BAR
        body.append(f'  <rect x="{bx}" y="{ry}" width="{w:.1f}" height="16" rx="4" fill="{fill}"/>')
        body.append(f'  <text x="{bx + w + 10:.1f}" y="{ry + 12}" class="{"val-hi" if hi else "val"}">{escape(label)}</text>')

    rx = 20 + pw + 14
    body.extend(card_box(rx, py, pw, ph, "Why yolowhtml Uses So Little Memory", "Designed for high-concurrency LLM & crawler pipelines"))
    bullets = [
        ("Zero-copy text:", "Slices point straight into input &str unless decoding entities."),
        ("Compact u32 DOM:", "One flat Vec<Node>; no per-node Box/Arc heap headers."),
        ("Single DOM build:", "Metadata, dates, main content & links read the same tree."),
        ("Arena recycling:", "process_with_workspace() reuses DOM & skipper buffers."),
    ]
    for i, (head, desc) in enumerate(bullets):
        by = py + 68 + i * 26
        body.append(f'  <circle cx="{rx + 22:.1f}" cy="{by - 4}" r="3" fill="{TEAL}"/>')
        body.append(f'  <text x="{rx + 32:.1f}" y="{by}" class="label-hi">{escape(head)}</text>')
        body.append(f'  <text x="{rx + 152:.1f}" y="{by}" class="note">{escape(desc)}</text>')

    (OUT / "memory-bench.svg").write_text(svg_doc(width, height, body), encoding="utf-8")


def make_architecture() -> None:
    width, height = 1040, 370
    body = header_band(
        width,
        "yolowhtml Architecture — Single-Pass Arena DOM to Clean Markdown",
        "Two focused MIT crates: yolowhtml-dom (HTML5 arena parser & CSS selectors) + yolowhtml (extraction & Markdown)",
        "Rust 2024 · MIT",
    )

    stages = [
        (
            "1. yolowhtml-dom",
            "Zero-Copy Arena HTML5",
            [
                "Single-pass tokenizer (memchr)",
                "Flat Vec<Node> + u32 NodeId",
                "HTML5 implicit tags & scopes",
                "Foster parenting & foreign SVG",
                "CSS selectors (:nth-child, :not)",
            ],
            INDIGO,
            INDIGO_SOFT,
            INDIGO_BORDER,
        ),
        (
            "2. Metadata & Dates",
            "Structured Header & Time",
            [
                "JSON-LD (@graph, Article, Video)",
                "OpenGraph, Twitter & meta tags",
                "20-language single-pass dates",
                "Preserves original timezones",
                "0.945 F1 on htmldate (0.31 ms)",
            ],
            BLUE,
            BLUE_SOFT,
            BLUE_BORDER,
        ),
        (
            "3. Main Content",
            "Hybrid Skipper & Arbitration",
            [
                "Nav/footer/ad/link-wall skipper",
                "Container vs Readability root",
                "50% prose share safety guard",
                "Auto relax: LinkBlocks -> All",
                "<noscript> & Video fallback",
            ],
            TEAL,
            TEAL_SOFT,
            TEAL_BORDER,
        ),
        (
            "4. GFM & Math",
            "1.000 F1 Markdown Renderer",
            [
                "Smart CommonMark escaping",
                "GFM tables & nested lists",
                "Code language class detection",
                "KaTeX / MathJax / Wiki TeX",
                "Built-in MathML -> LaTeX engine",
            ],
            EMERALD,
            EMERALD_SOFT,
            EMERALD_BORDER,
        ),
        (
            "5. Diagnostics",
            "Barriers, Links & Hashes",
            [
                "Paywall / CAPTCHA / Bot barrier",
                "Coverage completeness checks",
                "Resolved links & srcset images",
                "BLAKE3 + SimHash fingerprints",
                "JS API/link scanner (scan_script)",
            ],
            AMBER,
            AMBER_SOFT,
            AMBER_BORDER,
        ),
    ]

    sw = (width - 40 - 4 * 16) / 5
    sy, sh = 86, 196
    for i, (title, sub, bullets, col, bg, bd) in enumerate(stages):
        sx = 20 + i * (sw + 16)
        body.extend([
            f'  <rect x="{sx:.1f}" y="{sy}" width="{sw:.1f}" height="{sh}" rx="10" fill="{CARD}" stroke="{bd}" stroke-width="1.5" filter="url(#cardShadow)"/>',
            f'  <rect x="{sx:.1f}" y="{sy}" width="{sw:.1f}" height="48" rx="10" fill="{bg}"/>',
            f'  <rect x="{sx:.1f}" y="{sy + 38}" width="{sw:.1f}" height="10" fill="{bg}"/>',
            f'  <text x="{sx + 12:.1f}" y="{sy + 21}" class="label-hi" fill="{col}">{escape(title)}</text>',
            f'  <text x="{sx + 12:.1f}" y="{sy + 38}" class="kpi-sub">{escape(sub)}</text>',
        ])
        for j, b in enumerate(bullets):
            by = sy + 68 + j * 24
            body.append(f'  <circle cx="{sx + 15:.1f}" cy="{by - 4}" r="2.5" fill="{col}"/>')
            body.append(f'  <text x="{sx + 23:.1f}" y="{by}" class="note">{escape(b)}</text>')
        if i < 4:
            ax1 = sx + sw + 2
            ax2 = sx + sw + 14
            ay = sy + sh / 2
            body.append(f'  <line x1="{ax1:.1f}" y1="{ay:.1f}" x2="{ax2:.1f}" y2="{ay:.1f}" stroke="{INDIGO}" stroke-width="1.8" marker-end="url(#arrow)"/>')

    by = 298
    body.extend([
        f'  <rect x="20" y="{by}" width="{width - 40}" height="54" rx="10" fill="{CARD}" stroke="{BORDER}" filter="url(#cardShadow)"/>',
        f'  <text x="38" y="{by + 23}" class="label-hi">Unified Output: Document</text>',
        f'  <text x="215" y="{by + 23}" class="code">markdown · metadata · barrier · coverage · plain_text · html · links · images · fingerprints</text>',
        f'  <text x="38" y="{by + 42}" class="panel-sub">#![forbid(unsafe_code)] across both yolowhtml-dom and yolowhtml · Zero-allocation reuse via ProcessWorkspace · Powers PageSift 1.2.7+</text>',
    ])

    (OUT / "architecture.svg").write_text(svg_doc(width, height, body), encoding="utf-8")


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    make_hero_html_bench()
    make_quality_f1_bench()
    make_memory_bench()
    make_architecture()
    print("Generated 4 SVG figures in", OUT)


if __name__ == "__main__":
    main()
