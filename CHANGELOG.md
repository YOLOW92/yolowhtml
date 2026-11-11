# Changelog

## 0.1.0 — 2026-11-12

Initial standalone MIT release extracted from PageSift (`pagesift-html` and `pagesift-content`):

- **`yolowhtml-dom` (`0.1.0`)**:
  - Zero-copy arena HTML5 parser (`#![forbid(unsafe_code)]`) with contiguous `Vec<Node>` storage, `u32` node indices (`NodeId`), and `ParseScratch` arena reuse across worker invocations.
  - HTML5 tree-construction rules that affect content structure (implicit tags, auto-closing, foster parenting / table scopes, foreign SVG/MathML content, rawtext/rcdata, fragment and `scripting=false` modes).
  - Full HTML5 named entity decoding (`entities.rs`) with zero-copy cow slicing when text contains no `&`.
  - CSS selector engine (`Selector::parse`, `:nth-child`, `:not`, attribute and combinator matching).
- **`yolowhtml` (`0.1.0`)**:
  - Single-pass DOM main-content extraction (`process`, `process_with_workspace`, `process_with_workspace_then`): hybrid structural skipper + container/Readability dual-candidate arbitration + automatic fallback relaxation (`LinkBlocks` -> `All` -> `Hidden` -> `<noscript>` unwrap -> `VideoObject` JSON-LD).
  - High-fidelity GFM Markdown renderer (`1.000` mean F1 across 15 structural categories on 600 generated documents): context-aware CommonMark escaping, tables, nested lists, code blocks with language detection, and KaTeX / MathJax / Wikipedia / MathML → LaTeX formula recovery.
  - Multilingual publication-date extraction (`0.945` hold-out F1 on htmldate's 1,000-page benchmark at `0.31 ms/page`): JSON-LD, OpenGraph/meta, `<time>`, URL dates, and single-pass 20-language date scanners preserving original timezones.
  - Rich page metadata (`extract_metadata`), paywall / CAPTCHA / anti-bot barrier detection (`Barrier`, `BarrierKind`), content completeness warnings (`Coverage`), BLAKE3 + SimHash fingerprints, and JavaScript endpoint/link scanner (`scan_script`).
