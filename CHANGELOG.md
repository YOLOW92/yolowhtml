# Changelog

## 0.1.2 — 2026-11-18

- **Bounded input (`yolowhtml-dom` & `yolowhtml`)**: Source positions use 31 bits, so text past 2 GiB previously risked silent misreads. Inputs are now bounded at `MAX_INPUT_LEN` (1 GiB, after normalization) on a UTF-8 character boundary, decoded-text writes past the buffer limit are dropped cleanly, and `Html::is_truncated()` reports either case. `yolowhtml` surfaces truncation via `Coverage::input_truncated()` and an `input_truncated` warning.
- **HTML standard semantics (`yolowhtml`)**: Table `<caption>` elements are rendered as a leading paragraph before the GFM table (and permitted under strict article whitelisting); `hidden="until-found"` sections are kept as findable body content per HTML §6.1; `<q>` restores presentation-layer quotation marks (`“…”`, nested `‘…’`); `<menu>` and `<dir>` render as unordered lists in full-page mode.
- **Single-pass & article-mode speedups (`yolowhtml`, byte-identical output)**: `Skipper` caches default and strict-mode verdicts separately with a size-independent `IfSmall` state so toggling strict mode or switching candidate roots no longer clears the cache; text-node non-whitespace counts are recorded once during `Skipper` construction; Readability comma counting memoizes subtree totals bottom-up; `<h1>`/`<h2>` lookups in article mode use a lazy preorder interval index (`Outline`) recycled in `ProcessWorkspace` (`perf_gate` blog -19%, wiki -31%; AEB / WCXB / WebMainBench per-page time ratio `0.86` / `0.86` / `0.84`).
- **Cheaper `<noscript>` fallback (`yolowhtml`)**: Thin pages containing `<noscript>` blocks (GTM iframes, tracking pixels, "enable JavaScript" hints) now evaluate standalone `<noscript>` content length first and only re-parse the full page when unwrapping could reach the fallback threshold (`1.14` → `0.48 ms/page` on a 410 KB app shell with identical output).
- **Evaluation & conformance tooling**: Added `benchmarks/quality/` (`aeb.py`, `wcxb.py`, `webmain.py`, and paired bootstrap / randomization testing in `compare.py`), unified test/runtime environment variables under `YOLOWHTML_*`, and added `YOLOWHTML_HTML5LIB_REQUIRE=1` to fail instead of skipping when `html5lib-tests` data is absent.

## 0.1.1 — 2026-11-16

Fix silent dropping of `<figure>` and `<figcaption>` content in strict article extraction mode while keeping non-content noise filtering intact:

- **Article cleaning & furniture rules (`article.rs`, `extract.rs`, `features.rs`)**:
  - Removed `Tag::Figure` from `is_cleaned_tag()` and removed the unconditional `Tag::Figcaption => true` rule from `is_furniture()`, so legitimate figures and captions are no longer dropped solely by tag type.
  - Separated `caption` (`CAPTION`) and `attachment` (`ATTACHMENT`) class features from general `DISCARD` furniture so `<figure>` / `<figcaption>` elements carrying classes such as `wp-caption`, `wp-element-caption`, `figure-caption`, or `attachment` survive while `caption`-classed UI chrome outside `<figure>`/`<figcaption>` (and noise classes such as `ad`, `related`, `recommended`, `sponsored`, `sidebar`, `nav`, or hidden attributes on `<figure>`/`<figcaption>`) continue to be filtered.
- **Markdown whitelist & block rendering (`markdown/mod.rs`, `markdown/block.rs`)**:
  - Added `Tag::Figure` and `Tag::Figcaption` to `allows_paragraph()`, keeping direct images, direct text, inline formatting, links, formulas, code blocks, tables, blockquotes, and nested lists inside figures and captions.
  - Allowed `<div>` wrappers inside `<figcaption>` and rescued standalone non-text image/math inline wrappers inside `<figure>` under strict whitelisting without leaking stray `<div>` chrome.
- **Prose counting & arbitration consistency (`article.rs`)**:
  - Unified `article::prose()` with `markdown::allows_paragraph()` (including `editorial_attachment` and disclosure headers), ensuring candidate scoring, strict-share checks, and rendered Markdown stay aligned.

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
